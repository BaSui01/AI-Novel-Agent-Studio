use crate::providers::types::*;
use crate::types::{ChatMessage, ChatChoice, ChatCompletionResponse, TelemetryMetrics, UsageStats};
use std::collections::HashMap;

pub struct GeminiOptions {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Option<Vec<ChatCompletionTool>>,
    pub enable_thinking: bool,
    pub thinking_budget: Option<u32>,
    pub max_output_tokens: Option<u32>,
}

/// 转换为 Gemini inlineData 或 fileData (适配 Gemini API 纯 base64 要求，去除 prefix)
fn to_gemini_image_part(img: &ImageContent) -> serde_json::Value {
    let data = img.data.trim();

    if data.starts_with("http://") || data.starts_with("https://") {
        serde_json::json!({
            "fileData": {
                "mimeType": img.mime_type,
                "fileUri": data
            }
        })
    } else if data.starts_with("data:") && data.contains(";base64,") {
        let parts: Vec<&str> = data.split(";base64,").collect();
        let mime = parts[0].replace("data:", "");
        let base64_pure = parts.get(1).unwrap_or(&"");
        serde_json::json!({
            "inlineData": {
                "mimeType": if mime.is_empty() { &img.mime_type } else { &mime },
                "data": base64_pure
            }
        })
    } else {
        serde_json::json!({
            "inlineData": {
                "mimeType": img.mime_type,
                "data": data
            }
        })
    }
}

/// 转换为 Gemini functionDeclarations
pub fn convert_tools_to_gemini(tools: &[ChatCompletionTool]) -> Vec<serde_json::Value> {
    let decls: Vec<serde_json::Value> = tools
        .iter()
        .map(|t| {
            let params = &t.function.parameters;
            serde_json::json!({
                "name": t.function.name,
                "description": t.function.description,
                "parametersJsonSchema": {
                    "type": "object",
                    "properties": params.get("properties").cloned().unwrap_or(serde_json::json!({})),
                    "required": params.get("required").cloned().unwrap_or(serde_json::json!([]))
                }
            })
        })
        .collect();

    vec![serde_json::json!({ "functionDeclarations": decls })]
}

/// 构建 Gemini 原生 generateContent / streamGenerateContent 请求体
pub fn build_gemini_request_payload(opts: &GeminiOptions) -> serde_json::Value {
    let mut system_instruction: Option<serde_json::Value> = None;
    let mut contents: Vec<serde_json::Value> = Vec::new();

    // 用于 Tool Call ID 到函数名的映射，保证并行的 Tool Results 能精准对应 functionResponse
    let mut tool_id_to_name: HashMap<String, String> = HashMap::new();

    let mut i = 0;
    while i < opts.messages.len() {
        let msg = &opts.messages[i];

        if msg.role == "system" {
            system_instruction = Some(serde_json::json!({
                "parts": [{ "text": msg.content }]
            }));
            i += 1;
            continue;
        }

        if msg.role == "assistant" {
            let mut parts: Vec<serde_json::Value> = Vec::new();

            if let Some(ref thinking) = msg.thinking {
                parts.push(serde_json::json!({
                    "thought": true,
                    "text": thinking.thinking
                }));
            }

            if !msg.content.is_empty() {
                parts.push(serde_json::json!({ "text": msg.content }));
            }

            if let Some(ref calls) = msg.tool_calls {
                for call in calls {
                    tool_id_to_name.insert(call.id.clone(), call.function.name.clone());
                    let args_val: serde_json::Value = serde_json::from_str(&call.function.arguments)
                        .unwrap_or_else(|_| serde_json::json!({}));

                    let mut func_call_part = serde_json::json!({
                        "functionCall": {
                            "name": call.function.name,
                            "args": args_val
                        }
                    });

                    // 传递 thoughtSignature 解决 Gemini 思考模式下连续 Parallel Tool Call 的校验报错
                    if let Some(ref sig) = call.thought_signature {
                        func_call_part["thoughtSignature"] = serde_json::json!(sig);
                    }

                    parts.push(func_call_part);
                }
            }

            contents.push(serde_json::json!({
                "role": "model",
                "parts": parts
            }));
            i += 1;
            continue;
        }

        if msg.role == "tool" {
            // 连续收集连续的 tool message 形成单个 role: user 的 functionResponse content 块
            let mut response_parts: Vec<serde_json::Value> = Vec::new();

            while i < opts.messages.len() && opts.messages[i].role == "tool" {
                let tool_msg = &opts.messages[i];
                let func_name = tool_msg
                    .tool_call_id
                    .as_ref()
                    .and_then(|id| tool_id_to_name.get(id))
                    .cloned()
                    .unwrap_or_else(|| "unknown_function".to_string());

                response_parts.push(serde_json::json!({
                    "functionResponse": {
                        "name": func_name,
                        "response": {
                            "content": tool_msg.content
                        }
                    }
                }));
                i += 1;
            }

            contents.push(serde_json::json!({
                "role": "user",
                "parts": response_parts
            }));
            continue;
        }

        if msg.role == "user" {
            let mut parts: Vec<serde_json::Value> = Vec::new();
            if !msg.content.is_empty() {
                parts.push(serde_json::json!({ "text": msg.content }));
            }
            if let Some(ref imgs) = msg.images {
                for img in imgs {
                    parts.push(to_gemini_image_part(img));
                }
            }

            contents.push(serde_json::json!({
                "role": "user",
                "parts": parts
            }));
            i += 1;
        }
    }

    let mut generation_config = serde_json::json!({
        "temperature": opts.temperature.unwrap_or(0.7),
        "topP": opts.top_p.unwrap_or(0.95)
    });

    if opts.enable_thinking {
        if let Some(budget) = opts.thinking_budget {
            generation_config["thinkingConfig"] = serde_json::json!({
                "thinkingBudget": budget
            });
        }
    }

    if let Some(max_tokens) = opts.max_output_tokens {
        generation_config["maxOutputTokens"] = serde_json::json!(max_tokens);
    }

    let mut payload = serde_json::json!({
        "contents": contents,
        "generationConfig": generation_config
    });

    if let Some(sys) = system_instruction {
        payload["systemInstruction"] = sys;
    }

    if let Some(ref tools) = opts.tools {
        if !tools.is_empty() {
            payload["tools"] = serde_json::json!(convert_tools_to_gemini(tools));
        }
    }

    payload
}

// ─────────────────────────────────────────────────
//  SSE 流式解析辅助函数
// ─────────────────────────────────────────────────

/// 映射 Gemini finishReason → OpenAI finish_reason
pub fn map_gemini_finish_reason(reason: &str) -> &str {
    match reason {
        "STOP" => "stop",
        "MAX_TOKENS" => "length",
        "SAFETY" => "content_filter",
        "RECITATION" => "content_filter",
        "MALFORMED_FUNCTION_CALL" => "tool_calls",
        "BLOCKLIST" => "content_filter",
        "PROHIBITED_CONTENT" => "content_filter",
        "SPII" => "content_filter",
        "IMAGE_SAFETY" => "content_filter",
        _ => "stop",
    }
}

/// 从 Gemini SSE chunk 中提取 finishReason 并映射为 OpenAI finish_reason
pub fn extract_gemini_finish_reason(data_json: &serde_json::Value) -> Option<String> {
    data_json
        .pointer("/candidates/0/finishReason")
        .and_then(|f| f.as_str())
        .map(|r| map_gemini_finish_reason(r).to_string())
}

/// 从 Gemini SSE chunk 中遍历所有 parts，提取文本内容（连接所有text parts）
pub fn extract_gemini_text_delta(data_json: &serde_json::Value) -> Option<String> {
    let parts = data_json.pointer("/candidates/0/content/parts")
        .and_then(|p| p.as_array())?;
    
    let texts: Vec<&str> = parts.iter()
        .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
        .filter(|t| {
            // 排除 thought 标记的 parts（那些是 thinking，不是普通文本）
            !part_has_thought(parts.iter().find(|p| {
                p.get("text").and_then(|t| t.as_str()) == Some(t)
            }))
        })
        .collect();
    
    if texts.is_empty() { None } else { Some(texts.join("")) }
}

fn part_has_thought(part: Option<&serde_json::Value>) -> bool {
    part.and_then(|p| p.get("thought").and_then(|t| t.as_bool())).unwrap_or(false)
}

/// 从 Gemini SSE chunk 中提取 functionCall tool calls
pub fn extract_gemini_tool_calls_delta(data_json: &serde_json::Value) -> Option<Vec<ToolCall>> {
    let parts = data_json.pointer("/candidates/0/content/parts")
        .and_then(|p| p.as_array())?;
    
    let calls: Vec<ToolCall> = parts.iter()
        .filter_map(|part| part.get("functionCall"))
        .enumerate()
        .map(|(idx, fc)| {
            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let args = fc.get("args")
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_default();
            ToolCall {
                id: format!("call_{}_{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>(), idx),
                tool_type: "function".to_string(),
                function: FunctionCallDetails { name, arguments: args },
                thought_signature: None,
            }
        })
        .collect();
    
    if calls.is_empty() { None } else { Some(calls) }
}

/// 判断 Gemini 流式响应是否结束 (finishReason 存在且非空)
pub fn is_gemini_stream_end(data_json: &serde_json::Value) -> bool {
    data_json
        .pointer("/candidates/0/finishReason")
        .and_then(|f| f.as_str())
        .is_some()
}

/// 从 Gemini 流式响应中提取 usage metadata
pub fn extract_gemini_usage(data_json: &serde_json::Value) -> Option<(u32, u32)> {
    let prompt = data_json
        .pointer("/usageMetadata/promptTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let completion = data_json
        .pointer("/usageMetadata/candidatesTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    if prompt > 0 || completion > 0 {
        Some((prompt, completion))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────
//  非流式响应解析
// ─────────────────────────────────────────────────

/// 解析 Gemini generateContent 非流式响应 → OpenAI 兼容格式
pub fn parse_gemini_response(
    body: &serde_json::Value,
    model: &str,
    latency_ms: u64,
) -> ChatCompletionResponse {
    let parts = body
        .pointer("/candidates/0/content/parts")
        .and_then(|p| p.as_array());

    let mut content_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut thinking: Option<ThinkingInfo> = None;

    if let Some(parts_arr) = parts {
        for (idx, part) in parts_arr.iter().enumerate() {
            // Extract text
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                content_parts.push(text.to_string());
            }
            // Extract thought (thinking)
            if part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false) {
                if let Some(thought_text) = part.get("text").and_then(|t| t.as_str()) {
                    thinking = Some(ThinkingInfo {
                        thinking: thought_text.to_string(),
                        signature: None,
                    });
                }
            }
            // Extract functionCall
            if let Some(fc) = part.get("functionCall") {
                let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let args = fc.get("args")
                    .map(|a| serde_json::to_string(a).unwrap_or_default())
                    .unwrap_or_default();
                tool_calls.push(ToolCall {
                    id: format!("call_{}_{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>(), idx),
                    tool_type: "function".to_string(),
                    function: FunctionCallDetails { name, arguments: args },
                    thought_signature: part.get("thoughtSignature").and_then(|s| s.as_str()).map(|s| s.to_string()),
                });
            }
        }
    }

    let content = content_parts.join("");

    let finish_reason_raw = body
        .pointer("/candidates/0/finishReason")
        .and_then(|f| f.as_str())
        .unwrap_or("STOP");
    let finish_reason = map_gemini_finish_reason(finish_reason_raw);

    let prompt_tokens = body
        .pointer("/usageMetadata/promptTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let completion_tokens = body
        .pointer("/usageMetadata/candidatesTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let tps = if latency_ms > 0 {
        (completion_tokens as f64) / (latency_ms as f64 / 1000.0)
    } else {
        0.0
    };

    ChatCompletionResponse {
        id: format!("gemini-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: model.to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
                tool_call_id: None,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                images: None,
                thinking,
            },
            finish_reason: finish_reason.to_string(),
        }],
        usage: UsageStats {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
        metrics: TelemetryMetrics {
            latency_ms,
            ttft_ms: latency_ms.min(500),
            tps,
            cost_usd: 0.0,
        },
    }
}

/// 构建 Gemini 请求 Headers
pub fn build_gemini_headers(user_agent_str: &str) -> reqwest::header::HeaderMap {
    use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent_str).unwrap_or(HeaderValue::from_static("GeminiClient")));
    headers
}
