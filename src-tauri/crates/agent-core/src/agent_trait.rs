//! 通用 Agent Trait — 对标 snow-cli 中各 Agent 的 initialize/clearCache/isAvailable 模式。
//!
//! 所有 Agent 实现此 trait，由上层统一管理生命周期。

use gateway::client::{ChatOptions, GatewayClient};
use gateway::types::ChatMessage;
use gateway::GatewayError;

/// Agent 执行结果
pub type AgentResult<T> = Result<T, AgentError>;

/// Agent 层统一错误类型
#[derive(Debug, Clone)]
pub enum AgentError {
    /// 模型不可用（未配置 basic/advanced model）
    NotAvailable,
    /// 底层网关错误
    Gateway(String),
    /// 业务逻辑错误
    Business(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::NotAvailable => write!(f, "Agent 不可用：未配置模型"),
            AgentError::Gateway(msg) => write!(f, "网关错误: {}", msg),
            AgentError::Business(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<GatewayError> for AgentError {
    fn from(e: GatewayError) -> Self {
        AgentError::Gateway(format!("{:?}", e))
    }
}

/// Agent 配置（由调用方注入）
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// 使用的模型名（basicModel → 轻量任务，advancedModel → 复杂任务）
    pub model: Option<String>,
}

impl AgentConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: Some(model.into()),
        }
    }

    pub fn empty() -> Self {
        Self { model: None }
    }
}

/// 通用 Agent Trait
///
/// 实现此 trait 的结构体可直接调用 `GatewayClient` 与模型交互。
/// 对标 snow-cli 的 `initialize` → `isAvailable` → 调用模式。
pub trait Agent: Send + Sync {
    /// 懒初始化：校验并缓存当前配置
    fn initialize(&mut self, config: &AgentConfig) -> bool;

    /// 清除缓存配置（切换 Profile / 模型时调用）
    fn clear_cache(&mut self);

    /// 检查 Agent 是否就绪（首次调用时自动执行 initialize）
    fn is_available(&mut self, config: &AgentConfig) -> bool;

    /// Agent 标识名称
    fn name(&self) -> &'static str;
}

/// 辅助函数：用给定 model 和 messages 调用 GatewayClient 的非流式接口
pub async fn call_model(
    model: &str,
    messages: &[ChatMessage],
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) -> Result<String, AgentError> {
    let options = ChatOptions {
        temperature: temperature.or(Some(0.0)),
        max_tokens: max_tokens.or(Some(4096)),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        seed: None,
        stop: None,
        n: None,
        enable_thinking: false,
        thinking_budget_tokens: None,
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        response_format: None,
    };

    GatewayClient::chat_complete(model, messages, options)
        .await
        .map_err(AgentError::from)
}

/// 截断字符串到指定长度，超出部分加 "..."
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// 从模型文本响应中提取 JSON（自动处理 markdown 代码块包裹）
pub fn extract_json_from_response(response: &str) -> Option<serde_json::Value> {
    let trimmed = response.trim();

    // 尝试直接解析
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Some(val);
    }

    // 尝试从 markdown ```json ... ``` 块提取
    if let Some(start) = trimmed.find("```json") {
        let after_fence = &trimmed[start + 7..];
        if let Some(end) = after_fence.find("```") {
            let inner = after_fence[..end].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(inner) {
                return Some(val);
            }
        }
    }

    // 尝试从 ``` ... ``` 块提取
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        // 跳过可能的语言标识
        let content_start = after_fence.find('\n').map(|n| n + 1).unwrap_or(0);
        let content = &after_fence[content_start..];
        if let Some(end) = content.find("```") {
            let inner = content[..end].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(inner) {
                return Some(val);
            }
        }
    }

    // 尝试提取最外层 JSON 对象
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            let inner = &trimmed[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(inner) {
                return Some(val);
            }
        }
    }

    None
}
