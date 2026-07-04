use crate::providers::types::*;
use crate::types::{ChatMessage, ChatChoice, ChatCompletionResponse, TelemetryMetrics, UsageStats};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct AnthropicOptions {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ChatCompletionTool>>,
    pub session_id: Option<String>,
    pub enable_thinking: bool,
    pub thinking_budget_tokens: Option<u32>,
    pub enable_prompt_caching: bool,
}

/// Persistent User ID 计算 (对标 snow-cli getPersistentUserId)
/// 格式: user_<hash>_account__session_<uuid>
pub fn get_persistent_user_id(session_id: Option<&str>) -> String {
    let sid = session_id.map(|s| s.to_string()).unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut hasher = Sha256::new();
    hasher.update(format!("anthropic_user_{}", sid).as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("user_{}_account__session_{}", hash, sid)
}

/// 转换通用工具为 Anthropic 原生 Tool 格式
pub fn convert_tools_to_anthropic(tools: &[ChatCompletionTool]) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.function.name,
                "description": t.function.description,
                "input_schema": t.function.parameters
            })
        })
        .collect()
}

/// 将多模态图片转为 Anthropic Image Source
fn to_anthropic_image_source(img: &ImageContent) -> serde_json::Value {
    let data = img.data.trim();
    if data.starts_with("http://") || data.starts_with("https://") {
        serde_json::json!({
            "type": "url",
            "url": data
        })
    } else if data.starts_with("data:") && data.contains(";base64,") {
        let parts: Vec<&str> = data.split(";base64,").collect();
        let mime = parts[0].replace("data:", "");
        let base64_data = parts.get(1).unwrap_or(&"");
        serde_json::json!({
            "type": "base64",
            "media_type": if mime.is_empty() { &img.mime_type } else { &mime },
            "data": base64_data
        })
    } else {
        serde_json::json!({
            "type": "base64",
            "media_type": img.mime_type,
            "data": data
        })
    }
}

/// 构建 Anthropic 原生 /v1/messages 请求体
pub fn build_anthropic_request_payload(opts: &AnthropicOptions) -> serde_json::Value {
    let mut system_contents: Vec<serde_json::Value> = Vec::new();
    let mut anthropic_messages: Vec<serde_json::Value> = Vec::new();
    let mut tool_results: Vec<serde_json::Value> = Vec::new();

    for msg in &opts.messages {
        // 当遇到非 tool 消息时刷新积累的 tool_result 消息
        if msg.role != "tool" && !tool_results.is_empty() {
            anthropic_messages.push(serde_json::json!({
                "role": "user",
                "content": std::mem::take(&mut tool_results)
            }));
        }

        if msg.role == "system" {
            let mut block = serde_json::json!({
                "type": "text",
                "text": msg.content
            });
            if opts.enable_prompt_caching {
                block["cache_control"] = serde_json::json!({ "type": "ephemeral" });
            }
            system_contents.push(block);
            continue;
        }

        if msg.role == "tool" {
            if let Some(ref tool_call_id) = msg.tool_call_id {
                let mut content_array: Vec<serde_json::Value> = Vec::new();
                if !msg.content.is_empty() {
                    content_array.push(serde_json::json!({
                        "type": "text",
                        "text": msg.content
                    }));
                }
                if let Some(ref imgs) = msg.images {
                    for img in imgs {
                        content_array.push(serde_json::json!({
                            "type": "image",
                            "source": to_anthropic_image_source(img)
                        }));
                    }
                }

                let content_val = if content_array.len() == 1 && msg.images.as_ref().map_or(true, |i| i.is_empty()) {
                    serde_json::json!(msg.content)
                } else {
                    serde_json::json!(content_array)
                };

                tool_results.push(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": tool_call_id,
                    "content": content_val
                }));
            }
            continue;
        }

        if msg.role == "user" {
            let mut contents: Vec<serde_json::Value> = Vec::new();
            if !msg.content.is_empty() {
                contents.push(serde_json::json!({
                    "type": "text",
                    "text": msg.content
                }));
            }
            if let Some(ref imgs) = msg.images {
                for img in imgs {
                    contents.push(serde_json::json!({
                        "type": "image",
                        "source": to_anthropic_image_source(img)
                    }));
                }
            }

            anthropic_messages.push(serde_json::json!({
                "role": "user",
                "content": contents
            }));
            continue;
        }

        if msg.role == "assistant" {
            let mut contents: Vec<serde_json::Value> = Vec::new();

            if let Some(ref thinking) = msg.thinking {
                contents.push(serde_json::json!({
                    "type": "thinking",
                    "thinking": thinking.thinking,
                    "signature": thinking.signature
                }));
            }

            if !msg.content.is_empty() {
                contents.push(serde_json::json!({
                    "type": "text",
                    "text": msg.content
                }));
            }

            if let Some(ref calls) = msg.tool_calls {
                for call in calls {
                    let input_args: serde_json::Value = serde_json::from_str(&call.function.arguments)
                        .unwrap_or_else(|_| serde_json::json!({}));
                    contents.push(serde_json::json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.function.name,
                        "input": input_args
                    }));
                }
            }

            anthropic_messages.push(serde_json::json!({
                "role": "assistant",
                "content": contents
            }));
        }
    }

    if !tool_results.is_empty() {
        anthropic_messages.push(serde_json::json!({
            "role": "user",
            "content": tool_results
        }));
    }

    let mut payload = serde_json::json!({
        "model": opts.model,
        "max_tokens": opts.max_tokens.unwrap_or(4096),
        "messages": anthropic_messages,
        "stream": true
    });

    if !system_contents.is_empty() {
        payload["system"] = serde_json::json!(system_contents);
    }

    if let Some(ref tools) = opts.tools {
        if !tools.is_empty() {
            payload["tools"] = serde_json::json!(convert_tools_to_anthropic(tools));
        }
    }

    if let Some(temp) = opts.temperature {
        payload["temperature"] = serde_json::json!(temp);
    }
    if let Some(top_p) = opts.top_p {
        payload["top_p"] = serde_json::json!(top_p);
    }

    if opts.enable_thinking {
        payload["thinking"] = serde_json::json!({
            "type": "enabled",
            "budget_tokens": opts.thinking_budget_tokens.unwrap_or(2048)
        });
    }

    payload
}

/// 构建 Anthropic 请求 Headers
pub fn build_anthropic_headers(api_key: &str, user_agent_str: &str, enable_prompt_caching: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent_str).unwrap_or(HeaderValue::from_static("AnthropicClient")));
    if !api_key.is_empty() {
        if let Ok(val) = HeaderValue::from_str(api_key) {
            headers.insert("x-api-key", val);
        }
    }
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

    if enable_prompt_caching {
        headers.insert("anthropic-beta", HeaderValue::from_static("prompt-caching-2024-07-16"));
    }

    headers
}

// ─────────────────────────────────────────────────
//  SSE 流式解析辅助函数
// ─────────────────────────────────────────────────

/// 从 Anthropic SSE 事件中提取文本增量
pub fn extract_anthropic_text_delta(data_json: &serde_json::Value) -> Option<String> {
    let event_type = data_json.get("type").and_then(|t| t.as_str())?;
    match event_type {
        "content_block_delta" => data_json
            .pointer("/delta/text")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

/// 判断 Anthropic SSE 事件是否为流结束信号
pub fn is_anthropic_stream_end(data_json: &serde_json::Value) -> bool {
    let event_type = data_json.get("type").and_then(|t| t.as_str());
    matches!(event_type, Some("message_stop"))
}

/// 从 Anthropic SSE 事件中提取 usage
pub fn extract_anthropic_usage(data_json: &serde_json::Value) -> Option<(u32, u32)> {
    let event_type = data_json.get("type").and_then(|t| t.as_str())?;
    if event_type == "message_delta" {
        let output_tokens = data_json
            .pointer("/usage/output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        Some((0, output_tokens))
    } else if event_type == "message_start" {
        let input_tokens = data_json
            .pointer("/message/usage/input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        Some((input_tokens, 0))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────
//  非流式响应解析
// ─────────────────────────────────────────────────

/// 解析 Anthropic /v1/messages 非流式响应 → OpenAI 兼容格式
pub fn parse_anthropic_response(
    body: &serde_json::Value,
    latency_ms: u64,
) -> ChatCompletionResponse {
    let content = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| block.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let model = body
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("claude-unknown")
        .to_string();

    let input_tokens = body
        .pointer("/usage/input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = body
        .pointer("/usage/output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let stop_reason = body
        .get("stop_reason")
        .and_then(|s| s.as_str())
        .unwrap_or("end_turn");
    let finish_reason = match stop_reason {
        "end_turn" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool_calls",
        _ => "stop",
    };

    let tps = if latency_ms > 0 {
        (output_tokens as f64) / (latency_ms as f64 / 1000.0)
    } else {
        0.0
    };

    ChatCompletionResponse {
        id: body
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("msg-unknown")
            .to_string(),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
                tool_call_id: None,
                tool_calls: None,
                images: None,
                thinking: None,
            },
            finish_reason: finish_reason.to_string(),
        }],
        usage: UsageStats {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
        },
        metrics: TelemetryMetrics {
            latency_ms,
            ttft_ms: latency_ms.min(500),
            tps,
            cost_usd: 0.0,
        },
    }
}
