use serde::{Deserialize, Serialize};

/// 隐私脱敏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyMaskConfig {
    pub enabled: bool,
    pub mask_api_keys: bool,
    pub mask_bearer_tokens: bool,
    pub mask_file_paths: bool,
    pub custom_keywords: Vec<String>,
}

impl Default for PrivacyMaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mask_api_keys: true,
            mask_bearer_tokens: true,
            mask_file_paths: false,
            custom_keywords: Vec::new(),
        }
    }
}

/// 隐私掩码脱敏引擎 (参考 snow-cli privacyMask.ts)
pub struct PrivacyMasker;

impl PrivacyMasker {
    /// 对纯文本进行隐私脱敏替换
    pub fn mask_text(text: &str, config: &PrivacyMaskConfig) -> String {
        if !config.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();

        // 1. 脱敏 OpenAI / 通用 API Key (sk-...)
        if config.mask_api_keys {
            result = Self::mask_regex(&result, r"sk-[a-zA-Z0-9_-]{16,}", "sk-***[REDACTED_API_KEY]***");
            result = Self::mask_regex(&result, r"sk-ant-[a-zA-Z0-9_-]{16,}", "sk-ant-***[REDACTED_CLAUDE_KEY]***");
            result = Self::mask_regex(&result, r"AIzaSy[a-zA-Z0-9_-]{30,}", "AIzaSy***[REDACTED_GEMINI_KEY]***");
        }

        // 2. 脱敏 Bearer Token
        if config.mask_bearer_tokens {
            result = Self::mask_regex(&result, r"Bearer\s+eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+", "Bearer ***[REDACTED_TOKEN]***");
        }

        // 3. 脱敏自定义敏感词
        for kw in &config.custom_keywords {
            if !kw.is_empty() {
                let mask = "*".repeat(kw.chars().count());
                result = result.replace(kw, &mask);
            }
        }

        result
    }

    /// 对 ChatMessage 消息向量批量脱敏
    pub fn mask_messages(messages: &mut [crate::ChatMessage], config: &PrivacyMaskConfig) {
        if !config.enabled {
            return;
        }
        for msg in messages.iter_mut() {
            msg.content = Self::mask_text(&msg.content, config);
        }
    }

    fn mask_regex(text: &str, pattern: &str, replacement: &str) -> String {
        if let Ok(re) = regex::Regex::new(pattern) {
            re.replace_all(text, replacement).to_string()
        } else {
            text.to_string()
        }
    }
}
