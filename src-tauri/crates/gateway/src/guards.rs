use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

pub const DEFAULT_STREAM_IDLE_TIMEOUT_SEC: u64 = 180;
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const DEFAULT_RETRY_DELAY_MS: u64 = 1500;

/// 并发信号量限流保护器 (防止多 Agent 瞬间并发触发 429 报错)
pub static PROVIDER_SEMAPHORES: LazyLock<RwLock<HashMap<String, Arc<Semaphore>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct ProviderConcurrencyGuard;

impl ProviderConcurrencyGuard {
    pub fn get_semaphore(provider_name: &str, max_concurrency: usize) -> Arc<Semaphore> {
        if let Ok(guard) = PROVIDER_SEMAPHORES.read() {
            if let Some(sem) = guard.get(provider_name) {
                return sem.clone();
            }
        }

        let mut guard = PROVIDER_SEMAPHORES.write().unwrap();
        guard
            .entry(provider_name.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(max_concurrency)))
            .clone()
    }
}

/// Provider 主动熔断状态 (记录连续失败与冷却时间)
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub consecutive_failures: u32,
    pub cooldown_until: Option<Instant>,
}

pub static CIRCUIT_BREAKER: LazyLock<RwLock<HashMap<String, CircuitBreakerState>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct ProviderCircuitBreaker;

impl ProviderCircuitBreaker {
    pub const MAX_FAILURES_BEFORE_COOLING: u32 = 3;
    pub const COOLDOWN_DURATION_SEC: u64 = 60;

    /// 检查 Provider 是否正处于冷却熔断状态
    pub fn is_cooling_down(provider_name: &str) -> bool {
        if let Ok(guard) = CIRCUIT_BREAKER.read() {
            if let Some(state) = guard.get(provider_name) {
                if let Some(until) = state.cooldown_until {
                    if Instant::now() < until {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 记录一次成功调用 (重置连续失败计数与熔断状态)
    pub fn record_success(provider_name: &str) {
        if let Ok(mut guard) = CIRCUIT_BREAKER.write() {
            guard.remove(provider_name);
        }
    }

    /// 记录一次失败调用 (达到阀值进入 60s 熔断冷却)
    pub fn record_failure(provider_name: &str) {
        if let Ok(mut guard) = CIRCUIT_BREAKER.write() {
            let state = guard.entry(provider_name.to_string()).or_insert_with(|| CircuitBreakerState {
                consecutive_failures: 0,
                cooldown_until: None,
            });
            state.consecutive_failures += 1;
            if state.consecutive_failures >= Self::MAX_FAILURES_BEFORE_COOLING {
                state.cooldown_until = Some(Instant::now() + Duration::from_secs(Self::COOLDOWN_DURATION_SEC));
                eprintln!(
                    "[CircuitBreaker] Provider {} 连续失败 {} 次，触发熔断，冷却 60 秒!",
                    provider_name, state.consecutive_failures
                );
            }
        }
    }

    /// 持久化当前熔断器状态到磁盘
    pub fn save_to_disk(path: &str) {
        let state = CIRCUIT_BREAKER.read().unwrap().clone();
        let persisted: HashMap<String, PersistedBreakerState> = state
            .into_iter()
            .map(|(k, v)| (k, PersistedBreakerState {
                consecutive_failures: v.consecutive_failures,
            }))
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&persisted) {
            let temp = format!("{}.tmp", path);
            let _ = fs::write(&temp, &json);
            let _ = fs::rename(&temp, path);
        }
    }

    /// 从磁盘恢复熔断器状态（在启动时调用）
    pub fn load_from_disk(path: &str) {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<HashMap<String, PersistedBreakerState>>(&data) {
                let filtered: HashMap<String, CircuitBreakerState> = state
                    .into_iter()
                    .map(|(k, v)| (k, CircuitBreakerState {
                        consecutive_failures: v.consecutive_failures,
                        cooldown_until: None, // 重启后冷却计时器重置
                    }))
                    .collect();
                let mut cb = CIRCUIT_BREAKER.write().unwrap();
                *cb = filtered;
            }
        }
    }
}

/// 用于持久化的熔断器状态（不含 Instant）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedBreakerState {
    consecutive_failures: u32,
}

/// 对标 snow-cli parseJsonWithFix 的宽容 JSON 修复器 (用于自愈 Tool Calling 格式瑕疵)
pub fn parse_json_with_fix(raw_str: &str) -> serde_json::Value {
    let trimmed = raw_str.trim();
    if trimmed.is_empty() {
        return serde_json::json!({});
    }

    // 1. 尝试直接标准解析
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return val;
    }

    // 2. 自动修补单引号为双引号、移除尾随逗号、以及补充未闭合的大括号/中括号
    let mut fixed = trimmed.to_string();

    // 修复尾随逗号 e.g. {"a": 1,} -> {"a": 1}
    let re_trailing_comma = regex::Regex::new(r",\s*([\}\]])").unwrap();
    fixed = re_trailing_comma.replace_all(&fixed, "$1").to_string();

    // 补全缺失的闭合大括号
    let open_braces = fixed.chars().filter(|&c| c == '{').count();
    let close_braces = fixed.chars().filter(|&c| c == '}').count();
    if open_braces > close_braces {
        for _ in 0..(open_braces - close_braces) {
            fixed.push('}');
        }
    }

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&fixed) {
        return val;
    }

    serde_json::json!({ "raw": raw_str })
}

/// 重试与流防御引擎 (参考 snow-cli streamGuards.ts & retryUtils.ts)
pub struct GatewayGuard;

impl GatewayGuard {
    /// 判断 HTTP 状态码是否应该重试 (429 Rate Limit / 502 Bad Gateway / 503 Service Unavailable / 504 Gateway Timeout)
    pub fn is_retryable_status(status_code: u16) -> bool {
        matches!(status_code, 429 | 502 | 503 | 504)
    }

    /// 计算带退避策略的重试等待时长 (ms)
    pub fn calculate_backoff_delay(attempt: u32, base_delay_ms: u64) -> u64 {
        let factor = 2u64.pow(attempt.min(5));
        base_delay_ms * factor
    }

    /// 执行带自动指数重试的逻辑
    pub async fn execute_with_retry<F, Fut, T, E>(
        max_retries: u32,
        base_delay_ms: u64,
        operation: F,
    ) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(res) => return Ok(res),
                Err(err) => {
                    attempt += 1;
                    if attempt > max_retries {
                        return Err(err);
                    }
                    let delay = Self::calculate_backoff_delay(attempt, base_delay_ms);
                    eprintln!(
                        "[GatewayGuard] 操作失败 (第 {}/{} 次重试)，错误: {}，将在 {} ms 后重试...",
                        attempt, max_retries, err, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    /// 包装一个 SSE Stream，增加流式超时守护 (防止中途假死)
    pub fn wrap_stream_idle_guard<S, T, E>(
        stream: S,
        idle_timeout_sec: u64,
    ) -> impl futures_util::Stream<Item = Result<T, E>>
    where
        S: futures_util::Stream<Item = Result<T, E>> + Unpin,
        E: From<crate::GatewayError>,
    {
        use futures_util::StreamExt;
        let timeout_duration = Duration::from_secs(idle_timeout_sec);

        async_stream::stream! {
            let mut pinned = stream;
            loop {
                match tokio::time::timeout(timeout_duration, pinned.next()).await {
                    Ok(Some(item)) => {
                        yield item;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        yield Err(crate::GatewayError::NetworkTimeout(idle_timeout_sec * 1000).into());
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_with_fix_trailing_comma() {
        let malformed = r#"{"name": "test", "val": 123,}"#;
        let fixed = parse_json_with_fix(malformed);
        assert_eq!(fixed["name"], "test");
        assert_eq!(fixed["val"], 123);
    }

    #[test]
    fn test_parse_json_with_fix_unclosed_brace() {
        let malformed = r#"{"name": "test", "val": 123"#;
        let fixed = parse_json_with_fix(malformed);
        assert_eq!(fixed["name"], "test");
        assert_eq!(fixed["val"], 123);
    }

    #[test]
    fn test_circuit_breaker_cooling() {
        let p_name = "test_faulty_provider";
        assert!(!ProviderCircuitBreaker::is_cooling_down(p_name));

        ProviderCircuitBreaker::record_failure(p_name);
        ProviderCircuitBreaker::record_failure(p_name);
        assert!(!ProviderCircuitBreaker::is_cooling_down(p_name));

        ProviderCircuitBreaker::record_failure(p_name); // 3rd failure triggers cooling
        assert!(ProviderCircuitBreaker::is_cooling_down(p_name));

        ProviderCircuitBreaker::record_success(p_name);
        assert!(!ProviderCircuitBreaker::is_cooling_down(p_name));
    }
}
