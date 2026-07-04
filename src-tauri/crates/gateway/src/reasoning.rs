use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedReasoningContent {
    pub reasoning_content: Option<String>,
    pub main_content: String,
}

/// 思考过程提取与构建器 (针对 DeepSeek R1, Claude Extended Thinking, Gemini Thinking)
pub struct ReasoningExtractor;

impl ReasoningExtractor {
    /// 从文本中提取 <think>...</think> 思考内容
    pub fn extract_think_tags(text: &str) -> ParsedReasoningContent {
        if let (Some(start), Some(end)) = (text.find("<think>"), text.find("</think>")) {
            if start < end {
                let thinking = text[start + 7..end].trim().to_string();
                let main = format!("{}{}", &text[..start], &text[end + 8..]).trim().to_string();
                return ParsedReasoningContent {
                    reasoning_content: if thinking.is_empty() { None } else { Some(thinking) },
                    main_content: main,
                };
            }
        }

        ParsedReasoningContent {
            reasoning_content: None,
            main_content: text.to_string(),
        }
    }

    /// 构建同时支持 reasoning_content 和 content 的标准 OpenAI SSE Chunk
    pub fn build_reasoning_sse_chunk(
        req_id: &str,
        model: &str,
        reasoning_delta: Option<&str>,
        content_delta: Option<&str>,
        finish_reason: Option<&str>,
    ) -> serde_json::Value {
        let mut delta = serde_json::json!({});
        if let Some(r) = reasoning_delta {
            delta["reasoning_content"] = serde_json::json!(r);
        }
        if let Some(c) = content_delta {
            delta["content"] = serde_json::json!(c);
        }

        serde_json::json!({
            "id": req_id,
            "object": "chat.completion.chunk",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "delta": delta,
                "finish_reason": finish_reason
            }]
        })
    }
}
