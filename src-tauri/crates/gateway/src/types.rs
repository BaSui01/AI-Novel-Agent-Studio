use axum::{
    extract::{FromRequestParts, Json},
    http::request::Parts,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

pub use crate::i18n::{I18nManager, Language};
pub use crate::providers::types::{
    FunctionCallDetails, FunctionDeclaration, ImageContent, ThinkingInfo, ToolCall,
    UnifiedStreamChunk, UsageInfo,
};

/// 语言提取器：自动从 HTTP Header (Accept-Language / X-Language) 中提取客户端偏好语言
#[derive(Debug, Clone, Copy)]
pub struct RequestLang(pub &'static str);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequestLang
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let lang = parts
            .headers
            .get("x-language")
            .or_else(|| parts.headers.get("accept-language"))
            .and_then(|h| h.to_str().ok())
            .map(|s| {
                let l = s.to_lowercase();
                if l.contains("en") {
                    "en-US"
                } else if l.contains("ja") {
                    "ja-JP"
                } else {
                    "zh-CN"
                }
            })
            .unwrap_or("zh-CN");

        Ok(RequestLang(lang))
    }
}

/// 统一网关错误模型与国际化支持 (i18n: zh-CN / en-US / ja-JP)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GatewayError {
    ProviderNotFound(String),
    InvalidApiKey(String),
    UpstreamError { status_code: u16, message: String },
    NetworkTimeout(u64),
    RateLimitExceeded(u32),
    InvalidRequest(String),
    InternalError(String),
}

impl GatewayError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            GatewayError::ProviderNotFound(_) => axum::http::StatusCode::NOT_FOUND,
            GatewayError::InvalidApiKey(_) => axum::http::StatusCode::UNAUTHORIZED,
            GatewayError::UpstreamError { status_code, .. } => {
                axum::http::StatusCode::from_u16(*status_code).unwrap_or(axum::http::StatusCode::BAD_GATEWAY)
            }
            GatewayError::NetworkTimeout(_) => axum::http::StatusCode::GATEWAY_TIMEOUT,
            GatewayError::RateLimitExceeded(_) => axum::http::StatusCode::TOO_MANY_REQUESTS,
            GatewayError::InvalidRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            GatewayError::InternalError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_openai_json(&self, lang_str: &str) -> serde_json::Value {
        let lang = Language::from_str(lang_str);

        let (msg, code, provider) = match self {
            GatewayError::ProviderNotFound(p) => (
                I18nManager::tr(lang, "error.provider_not_found", &[("provider", p)]),
                "provider_not_found",
                Some(p.clone()),
            ),
            GatewayError::InvalidApiKey(p) => (
                I18nManager::tr(lang, "error.invalid_api_key", &[("provider", p)]),
                "invalid_api_key",
                Some(p.clone()),
            ),
            GatewayError::UpstreamError { status_code, message } => (
                I18nManager::tr(
                    lang,
                    "error.upstream_error",
                    &[
                        ("status_code", &status_code.to_string()),
                        ("message", message),
                    ],
                ),
                "upstream_error",
                None,
            ),
            GatewayError::NetworkTimeout(ms) => (
                I18nManager::tr(lang, "error.network_timeout", &[("ms", &ms.to_string())]),
                "network_timeout",
                None,
            ),
            GatewayError::RateLimitExceeded(secs) => (
                I18nManager::tr(lang, "error.rate_limit_exceeded", &[("secs", &secs.to_string())]),
                "rate_limit_exceeded",
                None,
            ),
            GatewayError::InvalidRequest(msg) => (
                I18nManager::tr(lang, "error.invalid_request", &[("message", msg)]),
                "invalid_request",
                None,
            ),
            GatewayError::InternalError(msg) => (
                I18nManager::tr(lang, "error.internal_error", &[("message", msg)]),
                "internal_error",
                None,
            ),
        };

        serde_json::json!({
            "error": {
                "message": msg,
                "type": "gateway_error",
                "param": null,
                "code": code,
                "provider": provider
            }
        })
    }

    pub fn into_response_with_lang(self, lang: &str) -> axum::response::Response {
        let status = self.status_code();
        let body = Json(self.to_openai_json(lang));
        (status, body).into_response()
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> axum::response::Response {
        self.into_response_with_lang("zh-CN")
    }
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for GatewayError {}

/// 支持的 Provider 类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    Custom,
}

/// 统一聊天消息定义 (整合基础角色文本与多模态、工具调用、思考过程)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant", "tool"
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<crate::providers::types::StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<crate::providers::types::ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<crate::providers::types::AudioConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<crate::providers::types::PredictionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<crate::providers::types::ChatCompletionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<crate::providers::types::ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryMetrics {
    pub latency_ms: u64,
    pub ttft_ms: u64,
    pub tps: f64,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: UsageStats,
    pub metrics: TelemetryMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetchUpstreamRequest {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestModelRequest {
    pub model_id: String,
    pub base_url: String,
    pub api_key: String,
    pub test_vision: bool,
}
