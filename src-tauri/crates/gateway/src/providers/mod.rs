pub mod anthropic;
pub mod gemini;
pub mod image_gen;
pub mod openai;
pub mod responses_api;
pub mod types;

pub use anthropic::*;
pub use gemini::*;
pub use image_gen::*;
pub use openai::*;
pub use responses_api::*;
pub use types::*;

/// 构建标准 OpenAI SSE chunk JSON (所有 provider 流式输出统一使用)
pub fn build_openai_sse_chunk(
    req_id: &str,
    model: &str,
    content: &str,
    finish_reason: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "id": req_id,
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "delta": {
                "content": content
            },
            "finish_reason": finish_reason
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChatMessage;

    #[test]
    fn test_anthropic_payload_with_prompt_caching() {
        let opts = AnthropicOptions {
            model: "claude-3-7-sonnet-20250219".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    tool_call_id: None,
                    tool_calls: None,
                    images: None,
                    thinking: None,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello!".to_string(),
                    tool_call_id: None,
                    tool_calls: None,
                    images: None,
                    thinking: None,
                },
            ],
            temperature: Some(0.7),
            top_p: None,
            max_tokens: Some(4096),
            tools: None,
            session_id: Some("session-123".to_string()),
            enable_thinking: true,
            thinking_budget_tokens: Some(2048),
            enable_prompt_caching: true,
        };

        let payload = build_anthropic_request_payload(&opts);
        assert_eq!(payload["model"], "claude-3-7-sonnet-20250219");
        assert_eq!(payload["thinking"]["type"], "enabled");
        assert_eq!(payload["thinking"]["budget_tokens"], 2048);
        assert_eq!(
            payload["system"][0]["cache_control"]["type"],
            "ephemeral"
        );
    }

    #[test]
    fn test_gemini_payload_with_thought_signature() {
        let opts = GeminiOptions {
            model: "gemini-2.5-flash".to_string(),
            messages: vec![
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "".to_string(),
                    tool_call_id: None,
                    tool_calls: Some(vec![ToolCall {
                        id: "call_abc".to_string(),
                        tool_type: "function".to_string(),
                        function: FunctionCallDetails {
                            name: "get_weather".to_string(),
                            arguments: r#"{"city":"Tokyo"}"#.to_string(),
                        },
                        thought_signature: Some("sig_12345".to_string()),
                    }]),
                    images: None,
                    thinking: Some(ThinkingInfo {
                        thinking: "Let me check the weather.".to_string(),
                        signature: None,
                    }),
                },
                ChatMessage {
                    role: "tool".to_string(),
                    content: "Sunny, 22C".to_string(),
                    tool_call_id: Some("call_abc".to_string()),
                    tool_calls: None,
                    images: None,
                    thinking: None,
                },
            ],
            temperature: Some(0.2),
            top_p: None,
            tools: None,
            enable_thinking: true,
            thinking_budget: Some(1024),
            max_output_tokens: None,
        };

        let payload = build_gemini_request_payload(&opts);
        let contents = payload["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0]["role"], "model");
        assert_eq!(
            contents[0]["parts"][1]["thoughtSignature"],
            "sig_12345"
        );

        assert_eq!(contents[1]["role"], "user");
        assert_eq!(
            contents[1]["parts"][0]["functionResponse"]["name"],
            "get_weather"
        );
    }

    #[test]
    fn test_image_gen_payloads() {
        let req = ImageGenRequest {
            model: "gpt-image-2".to_string(),
            prompt: "A beautiful sunset".to_string(),
            n: Some(1),
            size: Some("1024x1024".to_string()),
            quality: Some("hd".to_string()),
            response_format: Some("b64_json".to_string()),
            style: None,
            output_format: None,
            output_compression: None,
            background: None,
            moderation: None,
            user: None,
        };

        let openai_payload = build_openai_image_gen_payload(&req);
        assert_eq!(openai_payload["model"], "gpt-image-2");
        assert_eq!(openai_payload["quality"], "hd");

        let gemini_payload = build_gemini_image_gen_payload(&req);
        assert_eq!(
            gemini_payload["generationConfig"]["responseModalities"][1],
            "IMAGE"
        );
    }
}


