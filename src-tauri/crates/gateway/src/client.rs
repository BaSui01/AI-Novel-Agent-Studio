//! Gateway Client — 面向 Agent 的统一模型调用接口
//!
//! 将 chat handler 中的 Provider 路由、Payload 构建、流式解析等核心逻辑
//! 抽离为可复用 API，供 agent-core 等上游 crate 直接调用（无需走本地 HTTP 中转）。

use futures_util::Stream;
use std::pin::Pin;

use crate::dispatcher::GatewayDispatcher;
use crate::guards::{GatewayGuard, ProviderCircuitBreaker};
use crate::providers::{
    self, AnthropicOptions, GeminiOptions, OpenAIOptions,
};
use crate::types::{ChatMessage, GatewayError, ProviderType};
use crate::{SHARED_CLIENT, USER_AGENT_HEADER};

/// 模型调用选项
#[derive(Debug, Clone)]
pub struct ChatOptions {
    /// 采样温度 (None = 使用模型默认值)
    pub temperature: Option<f32>,
    /// 最大输出 token 数
    pub max_tokens: Option<u32>,
    /// Nucleus sampling (top_p)
    pub top_p: Option<f32>,
    /// 频率惩罚 (-2.0 ~ 2.0)
    pub frequency_penalty: Option<f32>,
    /// 存在惩罚 (-2.0 ~ 2.0)
    pub presence_penalty: Option<f32>,
    /// 随机种子
    pub seed: Option<i64>,
    /// 停止序列
    pub stop: Option<Vec<String>>,
    /// 返回 completion 数量
    pub n: Option<u32>,
    /// 是否启用思考过程 (Claude Extended Thinking / Gemini Thinking)
    pub enable_thinking: bool,
    /// 思考预算 tokens
    pub thinking_budget_tokens: Option<u32>,
    /// 工具定义 (Function Calling)
    pub tools: Option<Vec<crate::providers::types::ChatCompletionTool>>,
    /// Tool Choice 策略
    pub tool_choice: Option<crate::providers::types::ToolChoice>,
    /// 并行工具调用
    pub parallel_tool_calls: Option<bool>,
    /// Structured Output (JSON Schema)
    pub response_format: Option<crate::providers::types::ResponseFormat>,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.0),
            max_tokens: Some(4096),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            n: None,
            enable_thinking: false,
            thinking_budget_tokens: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            response_format: None,
        }
    }
}

/// 流式 chat completion 的单个事件
#[derive(Debug, Clone)]
pub enum ChatStreamEvent {
    /// 文本内容增量
    Content(String),
    /// 推理/思考过程增量
    ReasoningDelta(String),
    /// 工具调用
    ToolCalls(Vec<crate::providers::types::ToolCall>),
    /// Usage 信息 (通常在流末尾)
    Usage(crate::providers::types::UsageInfo),
    /// 流正常结束
    Done,
    /// 错误
    Error(String),
}

/// 统一模型调用客户端
///
/// # Example
/// ```ignore
/// use gateway::client::GatewayClient;
///
/// let events = GatewayClient::chat_stream(
///     "gpt-4o",
///     &[ChatMessage { role: "user".into(), content: "Hello".into(), ..Default::default() }],
///     ChatOptions::default(),
/// ).await?;
/// ```
pub struct GatewayClient;

impl GatewayClient {
    // ── 流式 Chat Completion ──────────────────────────────────────────

    /// 调用模型并以 `ChatStreamEvent` 流式返回结果。
    ///
    /// 自动进行 Provider 选择、Fallback 降级、CircuitBreaker 保护。
    pub async fn chat_stream(
        model: &str,
        messages: &[ChatMessage],
        options: ChatOptions,
    ) -> Result<
        Pin<Box<dyn Stream<Item = ChatStreamEvent> + Send>>,
        GatewayError,
    > {
        let (cfg, _) = crate::config::get_cached().await;
        let candidates = GatewayDispatcher::get_candidate_providers(&cfg, model);

        if candidates.is_empty() {
            return Err(GatewayError::ProviderNotFound(model.to_string()));
        }

        let mut last_error: Option<GatewayError> = None;

        for prov in candidates {
            let prov_type = GatewayDispatcher::detect_provider_type(prov);

            if prov.api_key.is_empty() && prov_type != ProviderType::Ollama {
                last_error = Some(GatewayError::InvalidApiKey(prov.display_name.clone()));
                continue;
            }

            let target_url =
                GatewayDispatcher::resolve_real_endpoint(prov, model, true, &prov_type);
            let client = SHARED_CLIENT.clone();
            let headers = Self::build_headers(&prov_type, &prov.api_key);
            let payload = Self::build_payload(model, messages, &options, &prov_type);

            let req_builder = client.post(&target_url).headers(headers);

            match req_builder.json(&payload).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let err_text = resp.text().await.unwrap_or_default();
                        ProviderCircuitBreaker::record_failure(&prov.name);

                        let status_code = status.as_u16();
                        if GatewayGuard::is_retryable_status(status_code) {
                            last_error =
                                Some(GatewayError::UpstreamError { status_code, message: err_text });
                            continue; // failover 到下一候选
                        } else {
                            return Err(GatewayError::UpstreamError {
                                status_code,
                                message: err_text,
                            });
                        }
                    }

                    ProviderCircuitBreaker::record_success(&prov.name);
                    let prov_type_clone = prov_type.clone();

                    let stream = async_stream::stream! {
                        let byte_stream = resp.bytes_stream();
                        futures_util::pin_mut!(byte_stream);
                        use futures_util::StreamExt;

                        let mut buffer = String::new();

                        while let Some(chunk_result) = byte_stream.next().await {
                            match chunk_result {
                                Ok(bytes) => {
                                    let chunk_str = String::from_utf8_lossy(&bytes);
                                    buffer.push_str(&chunk_str);

                                    while let Some(line_end) = buffer.find('\n') {
                                        let line = buffer[..line_end].trim().to_string();
                                        buffer.drain(..=line_end);

                                        if line.is_empty() {
                                            continue;
                                        }

                                        match prov_type_clone {
                                            ProviderType::Anthropic => {
                                                if let Some(data) = line.strip_prefix("data: ") {
                                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                                        if providers::is_anthropic_stream_end(&json) {
                                                            yield ChatStreamEvent::Done;
                                                            return;
                                                        }
                                                        if let Some(delta) = providers::extract_anthropic_text_delta(&json) {
                                                            yield ChatStreamEvent::Content(delta);
                                                        }
                                                    }
                                                }
                                            }
                                            ProviderType::Gemini => {
                                                if let Some(data) = line.strip_prefix("data: ") {
                                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                                        let is_end = providers::is_gemini_stream_end(&json);
                                                        if let Some(delta) = providers::extract_gemini_text_delta(&json) {
                                                            yield ChatStreamEvent::Content(delta);
                                                        }
                                                        if is_end {
                                                            yield ChatStreamEvent::Done;
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                // OpenAI / Ollama / Custom
                                                if let Some(data) = line.strip_prefix("data: ") {
                                                    if data == "[DONE]" {
                                                        yield ChatStreamEvent::Done;
                                                        return;
                                                    }
                                                    // 尝试解析 OpenAI SSE chunk
                                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                                        if let Some(choices) = json["choices"].as_array() {
                                                            if let Some(choice) = choices.first() {
                                                                if let Some(content) = choice["delta"]["content"].as_str() {
                                                                    yield ChatStreamEvent::Content(content.to_string());
                                                                }
                                                                // 推理内容 (DeepSeek R1 等)
                                                                if let Some(reasoning) = choice["delta"]["reasoning_content"].as_str() {
                                                                    if !reasoning.is_empty() {
                                                                        yield ChatStreamEvent::ReasoningDelta(reasoning.to_string());
                                                                    }
                                                                }
                                                                if choice["finish_reason"].as_str().is_some() {
                                                                    // finish_reason present → 即将结束
                                                                }
                                                            }
                                                        }
                                                        // 提取 usage
                                                        if let Some(usage) = json.get("usage") {
                                                            if let (Some(prompt), Some(completion)) =
                                                                (usage["prompt_tokens"].as_u64(), usage["completion_tokens"].as_u64())
                                                            {
                                                                yield ChatStreamEvent::Usage(
                                                                    crate::providers::types::UsageInfo {
                                                                        prompt_tokens: prompt as u32,
                                                                        completion_tokens: completion as u32,
                                                                        total_tokens: (prompt + completion) as u32,
                                                                    },
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        yield ChatStreamEvent::Done;
                    };

                    return Ok(Box::pin(stream));
                }
                Err(e) => {
                    ProviderCircuitBreaker::record_failure(&prov.name);
                    last_error = Some(GatewayError::UpstreamError {
                        status_code: 500,
                        message: format!("请求上游服务失败: {}", e),
                    });
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| GatewayError::ProviderNotFound(model.to_string())))
    }

    // ── 非流式 Chat Completion ────────────────────────────────────────

    /// 调用模型并返回完整文本响应。
    ///
    /// 内部使用流式 API 并组装完整结果。
    pub async fn chat_complete(
        model: &str,
        messages: &[ChatMessage],
        options: ChatOptions,
    ) -> Result<String, GatewayError> {
        use futures_util::StreamExt;

        let mut stream = Self::chat_stream(model, messages, options).await?;
        let mut content = String::new();

        while let Some(event) = stream.next().await {
            match event {
                ChatStreamEvent::Content(text) => content.push_str(&text),
                ChatStreamEvent::Done => break,
                ChatStreamEvent::Error(e) => return Err(GatewayError::UpstreamError {
                    status_code: 500,
                    message: e,
                }),
                _ => {} // 忽略推理/usage 等
            }
        }

        if content.is_empty() {
            return Err(GatewayError::InternalError("模型返回了空响应".to_string()));
        }

        Ok(content)
    }

    /// 调用模型并返回完整文本 + 推理内容
    pub async fn chat_complete_with_reasoning(
        model: &str,
        messages: &[ChatMessage],
        options: ChatOptions,
    ) -> Result<(String, Option<String>), GatewayError> {
        use futures_util::StreamExt;

        let mut stream = Self::chat_stream(model, messages, options).await?;
        let mut content = String::new();
        let mut reasoning = String::new();

        while let Some(event) = stream.next().await {
            match event {
                ChatStreamEvent::Content(text) => content.push_str(&text),
                ChatStreamEvent::ReasoningDelta(delta) => reasoning.push_str(&delta),
                ChatStreamEvent::Done => break,
                ChatStreamEvent::Error(e) => return Err(GatewayError::UpstreamError {
                    status_code: 500,
                    message: e,
                }),
                _ => {}
            }
        }

        let reasoning_opt = if reasoning.is_empty() { None } else { Some(reasoning) };

        Ok((content, reasoning_opt))
    }

    // ── 内部辅助 ──────────────────────────────────────────────────────

    fn build_headers(
        prov_type: &ProviderType,
        api_key: &str,
    ) -> reqwest::header::HeaderMap {
        match prov_type {
            ProviderType::Anthropic => {
                providers::build_anthropic_headers(api_key, USER_AGENT_HEADER, true)
            }
            ProviderType::Gemini => providers::build_gemini_headers(USER_AGENT_HEADER),
            _ => providers::build_openai_headers(api_key, USER_AGENT_HEADER, None),
        }
    }

    fn build_payload(
        model: &str,
        messages: &[ChatMessage],
        options: &ChatOptions,
        prov_type: &ProviderType,
    ) -> serde_json::Value {
        match prov_type {
            ProviderType::Anthropic => {
                let opts = AnthropicOptions {
                    model: model.to_string(),
                    messages: messages.to_vec(),
                    temperature: options.temperature,
                    top_p: options.top_p,
                    max_tokens: options.max_tokens,
                    tools: options.tools.clone(),
                    session_id: None,
                    enable_thinking: options.enable_thinking,
                    thinking_budget_tokens: options.thinking_budget_tokens,
                    enable_prompt_caching: false,
                };
                providers::build_anthropic_request_payload(&opts)
            }
            ProviderType::Gemini => {
                let opts = GeminiOptions {
                    model: model.to_string(),
                    messages: messages.to_vec(),
                    temperature: options.temperature,
                    top_p: options.top_p,
                    tools: options.tools.clone(),
                    enable_thinking: options.enable_thinking,
                    thinking_budget: options.thinking_budget_tokens,
                    max_output_tokens: options.max_tokens,
                };
                providers::build_gemini_request_payload(&opts)
            }
            _ => {
                let opts = OpenAIOptions {
                    model: model.to_string(),
                    messages: messages.to_vec(),
                    temperature: options.temperature,
                    max_tokens: options.max_tokens,
                    top_p: options.top_p,
                    frequency_penalty: options.frequency_penalty,
                    presence_penalty: options.presence_penalty,
                    seed: options.seed,
                    stop: options.stop.clone(),
                    n: options.n,
                    logprobs: None,
                    top_logprobs: None,
                    stream: Some(true),
                    stream_options: None,
                    response_format: options.response_format.clone(),
                    modalities: None,
                    audio: None,
                    prediction: None,
                    tools: options.tools.clone(),
                    tool_choice: options.tool_choice.clone(),
                    parallel_tool_calls: options.parallel_tool_calls,
                    custom_headers: None,
                };
                providers::build_openai_request_payload(&opts)
            }
        }
    }
}
