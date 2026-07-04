use crate::types::ProviderType;
use crate::config::{GatewayConfig, ProviderItem};

pub struct GatewayDispatcher;

impl GatewayDispatcher {
    /// 根据中转站 Base URL 或自定义端点自动构建实际请求目标 URL
    pub fn resolve_target_url(provider: &ProviderItem, model: &str, is_responses_api: bool) -> String {
        let base = provider.base_url.trim_end_matches('/');

        if provider.name == "gemini" {
            if let Some(ref compat) = provider.openai_compat_url {
                if !compat.is_empty() {
                    return format!("{}/chat/completions", compat.trim_end_matches('/'));
                }
            }
            return format!("{}/models/{}:generateContent?key={}", base, model, provider.api_key);
        }

        if is_responses_api {
            if let Some(ref resp_url) = provider.responses_url {
                if !resp_url.is_empty() {
                    return resp_url.clone();
                }
            }
            return format!("{}/responses", base);
        }

        if let Some(ref chat_url) = provider.chat_completions_url {
            if !chat_url.is_empty() {
                return chat_url.clone();
            }
        }

        // 默认自由中转站 Base URL 拼接: {custom_base_url}/chat/completions
        format!("{}/chat/completions", base)
    }

    /// 校验与选择有效的 Provider (优先 Active，失败降级到 Fallback 链)
    pub fn select_provider<'a>(cfg: &'a GatewayConfig, requested_model: &str) -> Option<&'a ProviderItem> {
        // 1. 尝试从包含该模型的已启用 Provider 中查找
        for p in &cfg.providers {
            if p.is_enabled && p.models.iter().any(|m| m.id.eq_ignore_ascii_case(requested_model)) {
                return Some(p);
            }
        }

        // 2. 按 Fallback 链回退选择第一个已启用的 Provider
        for name in &cfg.fallback_chain {
            if let Some(p) = cfg.providers.iter().find(|p| p.is_enabled && p.name == *name) {
                return Some(p);
            }
        }

        // 3. 回退选择第一个已启用的中转站配置
        cfg.providers.iter().find(|p| p.is_enabled)
    }

    /// 自动检测 Provider 类型 (参考 snow-cli endpointResolver 逻辑)
    /// 根据 provider name 和 base_url 综合判定
    pub fn detect_provider_type(provider: &ProviderItem) -> ProviderType {
        let name = provider.name.to_lowercase();
        let base = provider.base_url.to_lowercase();

        // 精确匹配 provider name
        if name == "anthropic" || name == "claude" {
            return ProviderType::Anthropic;
        }
        if name == "gemini" || name == "google" {
            return ProviderType::Gemini;
        }
        if name == "ollama" {
            return ProviderType::Ollama;
        }
        if name == "openai" {
            return ProviderType::OpenAI;
        }

        // 通过 base_url 特征判断
        if base.contains("anthropic.com") {
            return ProviderType::Anthropic;
        }
        if base.contains("generativelanguage.googleapis.com") {
            return ProviderType::Gemini;
        }
        if base.contains("openai.com") {
            return ProviderType::OpenAI;
        }
        if base.contains("127.0.0.1:11434") || base.contains("localhost:11434") {
            return ProviderType::Ollama;
        }

        // 如果有 OpenAI 兼容 URL，视为 OpenAI 兼容
        if provider.openai_compat_url.is_some() {
            return ProviderType::OpenAI;
        }

        // 默认视为 OpenAI 兼容中转站
        ProviderType::Custom
    }

    /// 根据 Provider 类型和流式模式，构建真实的请求 URL
    pub fn resolve_real_endpoint(
        provider: &ProviderItem,
        model: &str,
        is_stream: bool,
        provider_type: &ProviderType,
    ) -> String {
        let base = provider.base_url.trim_end_matches('/');

        match provider_type {
            ProviderType::Anthropic => {
                // Anthropic: {base_url}/messages
                format!("{}/messages", base)
            }
            ProviderType::Gemini => {
                // 优先使用 OpenAI 兼容端点
                if let Some(ref compat) = provider.openai_compat_url {
                    if !compat.is_empty() {
                        return format!("{}/chat/completions", compat.trim_end_matches('/'));
                    }
                }
                // 原生 Gemini REST API
                let action = if is_stream {
                    "streamGenerateContent?alt=sse"
                } else {
                    "generateContent"
                };
                format!(
                    "{}/models/{}:{}?key={}",
                    base, model, action, provider.api_key
                )
            }
            ProviderType::Ollama => {
                // Ollama OpenAI 兼容: {base_url}/v1/chat/completions
                if base.ends_with("/v1") {
                    format!("{}/chat/completions", base)
                } else {
                    format!("{}/v1/chat/completions", base)
                }
            }
            ProviderType::OpenAI | ProviderType::Custom => {
                // 优先使用明确配置的 chat_completions_url
                if let Some(ref chat_url) = provider.chat_completions_url {
                    if !chat_url.is_empty() {
                        return chat_url.clone();
                    }
                }
                format!("{}/chat/completions", base)
            }
        }
    }

    /// 获取备选 Provider 链列表 (首选匹配 -> Fallback 链 -> 任何启用配置)
    pub fn get_candidate_providers<'a>(
        cfg: &'a GatewayConfig,
        requested_model: &str,
    ) -> Vec<&'a ProviderItem> {
        let mut candidates = Vec::new();

        // 1. 明确匹配模型的已启用 Provider (排除熔断冷却中)
        for p in &cfg.providers {
            if p.is_enabled && !crate::guards::ProviderCircuitBreaker::is_cooling_down(&p.name) && p.models.iter().any(|m| m.id.eq_ignore_ascii_case(requested_model)) {
                if !candidates.iter().any(|c: &&ProviderItem| c.name == p.name) {
                    candidates.push(p);
                }
            }
        }

        // 2. 依次加入 Fallback 链配置 (排除熔断冷却中)
        for name in &cfg.fallback_chain {
            if let Some(p) = cfg.providers.iter().find(|p| p.is_enabled && !crate::guards::ProviderCircuitBreaker::is_cooling_down(&p.name) && p.name == *name) {
                if !candidates.iter().any(|c: &&ProviderItem| c.name == p.name) {
                    candidates.push(p);
                }
            }
        }

        // 3. 加入其余任意已启用的中转站
        for p in &cfg.providers {
            if p.is_enabled && !candidates.iter().any(|c: &&ProviderItem| c.name == p.name) {
                candidates.push(p);
            }
        }

        candidates
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModelInfo;

    #[test]
    fn test_candidate_providers_fallback_ordering() {
        let cfg = GatewayConfig {
            providers: vec![
                ProviderItem {
                    name: "openai".to_string(),
                    display_name: "OpenAI".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    api_key: "sk-1".to_string(),
                    is_enabled: true,
                    models: vec![ModelInfo::create_default("gpt-4o")],
                    chat_completions_url: None,
                    responses_url: None,
                    openai_compat_url: None,
                },
                ProviderItem {
                    name: "anthropic".to_string(),
                    display_name: "Anthropic".to_string(),
                    base_url: "https://api.anthropic.com/v1".to_string(),
                    api_key: "sk-2".to_string(),
                    is_enabled: true,
                    models: vec![ModelInfo::create_default("claude-3-7-sonnet")],
                    chat_completions_url: None,
                    responses_url: None,
                    openai_compat_url: None,
                },
            ],
            fallback_chain: vec!["anthropic".to_string(), "openai".to_string()],
            active_provider: "openai".to_string(),
        };


        let candidates = GatewayDispatcher::get_candidate_providers(&cfg, "claude-3-7-sonnet");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].name, "anthropic");
        assert_eq!(candidates[1].name, "openai");
    }
}


