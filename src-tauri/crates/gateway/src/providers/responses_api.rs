use crate::providers::types::*;
use crate::types::ChatMessage;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};

pub struct ResponsesApiOptions {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ChatCompletionTool>>,
}

/// 构建 OpenAI /v1/responses 请求 Payload (对标 snow-cli/source/api/responses.ts)
pub fn build_responses_api_payload(opts: &ResponsesApiOptions) -> serde_json::Value {
    let mut instructions = String::new();
    let mut input_messages: Vec<serde_json::Value> = Vec::new();

    for m in &opts.messages {
        if m.role == "system" {
            if !instructions.is_empty() {
                instructions.push('\n');
            }
            instructions.push_str(&m.content);
        } else {
            input_messages.push(serde_json::json!({
                "role": m.role,
                "content": m.content
            }));
        }
    }

    let mut payload = serde_json::json!({
        "model": opts.model,
        "input": input_messages,
        "stream": true
    });

    if !instructions.is_empty() {
        payload["instructions"] = serde_json::json!(instructions);
    }
    if let Some(temp) = opts.temperature {
        payload["temperature"] = serde_json::json!(temp);
    }
    if let Some(max_t) = opts.max_tokens {
        payload["max_output_tokens"] = serde_json::json!(max_t);
    }
    if let Some(ref tools) = opts.tools {
        if !tools.is_empty() {
            payload["tools"] = serde_json::json!(tools);
        }
    }

    payload
}

pub fn build_responses_api_headers(api_key: &str, user_agent_str: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent_str).unwrap_or(HeaderValue::from_static("ResponsesApiClient")));

    if !api_key.is_empty() {
        let auth_val = format!("Bearer {}", api_key);
        if let Ok(val) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, val);
        }
    }

    headers
}
