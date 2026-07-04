use crate::providers::types::*;
use crate::types::ChatMessage;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use std::collections::HashMap;

pub struct OpenAIOptions {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<i64>,
    pub stop: Option<Vec<String>>,
    pub n: Option<u32>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u32>,
    pub stream: Option<bool>,
    pub stream_options: Option<crate::providers::types::StreamOptions>,
    pub response_format: Option<crate::providers::types::ResponseFormat>,
    pub modalities: Option<Vec<String>>,
    pub audio: Option<crate::providers::types::AudioConfig>,
    pub prediction: Option<crate::providers::types::PredictionConfig>,
    pub tools: Option<Vec<ChatCompletionTool>>,
    pub tool_choice: Option<crate::providers::types::ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    pub custom_headers: Option<HashMap<String, String>>,
}

/// 转换为 OpenAI 多模态 Image Content
fn to_openai_image_url(img: &ImageContent) -> serde_json::Value {
    let data = img.data.trim();
    let url_str = if data.starts_with("http://") || data.starts_with("https://") || data.starts_with("data:") {
        data.to_string()
    } else {
        format!("data:{};base64,{}", img.mime_type, data)
    };

    serde_json::json!({
        "type": "image_url",
        "image_url": {
            "url": url_str
        }
    })
}

/// 构建 OpenAI / OpenAI 兼容中转站请求 Payload
pub fn build_openai_request_payload(opts: &OpenAIOptions) -> serde_json::Value {
    let messages: Vec<serde_json::Value> = opts
        .messages
        .iter()
        .map(|m| {
            if m.role == "user" {
                if let Some(ref imgs) = m.images {
                    if !imgs.is_empty() {
                        let mut contents = vec![serde_json::json!({
                            "type": "text",
                            "text": m.content
                        })];
                        for img in imgs {
                            contents.push(to_openai_image_url(img));
                        }
                        return serde_json::json!({
                            "role": "user",
                            "content": contents
                        });
                    }
                }
                serde_json::json!({
                    "role": "user",
                    "content": m.content
                })
            } else if m.role == "assistant" {
                let mut node = serde_json::json!({
                    "role": "assistant",
                    "content": m.content
                });
                if let Some(ref calls) = m.tool_calls {
                    node["tool_calls"] = serde_json::json!(calls);
                }
                if let Some(ref thinking) = m.thinking {
                    node["reasoning_content"] = serde_json::json!(thinking.thinking);
                }
                node
            } else if m.role == "tool" {
                serde_json::json!({
                    "role": "tool",
                    "tool_call_id": m.tool_call_id.as_deref().unwrap_or(""),
                    "content": m.content
                })
            } else {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            }
        })
        .collect();

    let mut payload = serde_json::json!({
        "model": opts.model,
        "messages": messages,
        "stream": opts.stream.unwrap_or(true),
    });

    if opts.stream.unwrap_or(true) {
        if let Some(ref so) = opts.stream_options {
            payload["stream_options"] = serde_json::json!(so);
        } else {
            payload["stream_options"] = serde_json::json!({ "include_usage": true });
        }
    }

    if let Some(temp) = opts.temperature {
        payload["temperature"] = serde_json::json!(temp);
    }
    if let Some(max_t) = opts.max_tokens {
        payload["max_tokens"] = serde_json::json!(max_t);
    }
    if let Some(top_p) = opts.top_p {
        payload["top_p"] = serde_json::json!(top_p);
    }
    if let Some(fp) = opts.frequency_penalty {
        payload["frequency_penalty"] = serde_json::json!(fp);
    }
    if let Some(pp) = opts.presence_penalty {
        payload["presence_penalty"] = serde_json::json!(pp);
    }
    if let Some(s) = opts.seed {
        payload["seed"] = serde_json::json!(s);
    }
    if let Some(ref stop) = opts.stop {
        if !stop.is_empty() {
            payload["stop"] = serde_json::json!(stop);
        }
    }
    if let Some(n) = opts.n {
        payload["n"] = serde_json::json!(n);
    }
    if let Some(lp) = opts.logprobs {
        payload["logprobs"] = serde_json::json!(lp);
    }
    if let Some(tlp) = opts.top_logprobs {
        payload["top_logprobs"] = serde_json::json!(tlp);
    }
    if let Some(ref rf) = opts.response_format {
        payload["response_format"] = serde_json::json!(rf);
    }
    if let Some(ref mods) = opts.modalities {
        payload["modalities"] = serde_json::json!(mods);
    }
    if let Some(ref audio) = opts.audio {
        payload["audio"] = serde_json::json!(audio);
    }
    if let Some(ref pred) = opts.prediction {
        payload["prediction"] = serde_json::json!(pred);
    }
    if let Some(ref tools) = opts.tools {
        if !tools.is_empty() {
            payload["tools"] = serde_json::json!(tools);
        }
    }
    if let Some(ref tc) = opts.tool_choice {
        payload["tool_choice"] = serde_json::json!(tc);
    }
    if let Some(ptc) = opts.parallel_tool_calls {
        payload["parallel_tool_calls"] = serde_json::json!(ptc);
    }

    payload
}

/// 构建 OpenAI 请求 Headers
pub fn build_openai_headers(api_key: &str, user_agent_str: &str, custom: Option<&HashMap<String, String>>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent_str).unwrap_or(HeaderValue::from_static("OpenAIClient")));

    if !api_key.is_empty() {
        let auth_val = format!("Bearer {}", api_key);
        if let Ok(val) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, val);
        }
    }

    if let Some(custom_map) = custom {
        for (k, v) in custom_map {
            if let (Ok(name), Ok(val)) = (reqwest::header::HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(v)) {
                headers.insert(name, val);
            }
        }
    }

    headers
}
