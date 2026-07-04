use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesApiRequest {
    pub model: String,
    pub input: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ResponsesReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesReasoningConfig {
    pub effort: String, // "none" | "low" | "medium" | "high" | "xhigh"
}

/// OpenAI 2025/2026 Responses API 适配器 (参考 snow-cli responses.ts)
pub struct ResponsesApiAdapter;

impl ResponsesApiAdapter {
    /// 递归确保 JSON Schema 符合 Responses API 的 Strict Schema 要求 (additionalProperties: false)
    pub fn ensure_strict_schema(schema: &mut serde_json::Value) {
        if let Some(obj) = schema.as_object_mut() {
            if obj.get("type").and_then(|t| t.as_str()) == Some("object") {
                obj.insert("additionalProperties".to_string(), serde_json::json!(false));
            }
            if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
                for (_k, v) in props.iter_mut() {
                    Self::ensure_strict_schema(v);
                }
            }
        }
    }

    /// 将通用 ChatMessage 转化为 Responses API 规定的 input 数组结构
    pub fn convert_messages_to_responses_input(messages: &[crate::ChatMessage]) -> (Vec<serde_json::Value>, Option<String>) {
        let mut input = Vec::new();
        let mut instructions_acc = String::new();

        for m in messages {
            if m.role == "system" {
                if !instructions_acc.is_empty() {
                    instructions_acc.push('\n');
                }
                instructions_acc.push_str(&m.content);
            } else if m.role == "user" {
                input.push(serde_json::json!({
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": m.content
                    }]
                }));
            } else if m.role == "assistant" {
                input.push(serde_json::json!({
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": m.content
                    }]
                }));
            }
        }

        let instructions = if instructions_acc.is_empty() {
            None
        } else {
            Some(instructions_acc)
        };

        (input, instructions)
    }
}
