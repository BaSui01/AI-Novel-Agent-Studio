use axum::{
    extract::{Json, Path},
    response::sse::{Event, Sse},
    response::IntoResponse,
};
use futures_util::StreamExt;
use std::convert::Infallible;
use std::time::Instant;

use crate::dispatcher::GatewayDispatcher;
use crate::guards::{GatewayGuard, ProviderCircuitBreaker};
use crate::privacy_mask::{PrivacyMaskConfig, PrivacyMasker};
use crate::providers::{
    build_anthropic_headers, build_anthropic_request_payload, build_gemini_headers,
    build_gemini_request_payload, build_openai_headers, build_openai_request_payload,
    build_openai_sse_chunk, extract_anthropic_text_delta, extract_gemini_finish_reason,
    extract_gemini_text_delta, extract_gemini_tool_calls_delta, extract_gemini_usage,
    is_anthropic_stream_end, is_gemini_stream_end, parse_anthropic_response,
    parse_gemini_response, AnthropicOptions, GeminiOptions, OpenAIOptions,
};
use crate::responses_api::ResponsesApiAdapter;
use crate::types::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, GatewayError, ProviderType,
    RequestLang, TelemetryMetrics, UsageStats,
};
use crate::{SHARED_CLIENT, USER_AGENT_HEADER};

/// 处理 `/v1/chat/completions` 核心入口 — 具备 Candidate Fallback 降级与 CircuitBreaker 状态闭环
pub async fn handle_chat_completions(
    RequestLang(lang): RequestLang,
    Json(mut req): Json<ChatCompletionRequest>,
) -> axum::response::Response {
    // 自动应用隐私脱敏处理
    let mask_config = PrivacyMaskConfig::default();
    PrivacyMasker::mask_messages(&mut req.messages, &mask_config);

    let (cfg, index) = crate::config::get_cached().await;
    let model_item = cfg.find_model(&index, &req.model).cloned();

    // 1. 获取包含模型优先与 Fallback 链排名的 Candidate 列表
    let candidates = GatewayDispatcher::get_candidate_providers(&cfg, &req.model);
    if candidates.is_empty() {
        return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang);
    }

    let mut last_error: Option<GatewayError> = None;

    // 2. 依次尝试候选 Provider (实现失败自动 Failover 无感切节点)
    for prov in candidates {
        // 校验 API Key (Ollama 及无需密中转站除外)
        let prov_type = GatewayDispatcher::detect_provider_type(prov);
        if prov.api_key.is_empty() && prov_type != ProviderType::Ollama {
            last_error = Some(GatewayError::InvalidApiKey(prov.display_name.clone()));
            continue;
        }

        let target_url = GatewayDispatcher::resolve_real_endpoint(prov, &req.model, req.stream, &prov_type);
        let client = SHARED_CLIENT.clone();
        let mut req_builder = client.post(&target_url);

        // 构建 Headers
        let headers = match prov_type {
            ProviderType::Anthropic => build_anthropic_headers(&prov.api_key, USER_AGENT_HEADER, true),
            ProviderType::Gemini => build_gemini_headers(USER_AGENT_HEADER),
            _ => build_openai_headers(&prov.api_key, USER_AGENT_HEADER, None),
        };
        req_builder = req_builder.headers(headers);

        // 构建 Payload (使用 providers 模块的高级构造器，全面支持 Prompt Caching、Thinking、Tool Calling 和多模态)
        let payload = match prov_type {
            ProviderType::Anthropic => {
                let opts = AnthropicOptions {
                    model: req.model.clone(),
                    messages: req.messages.clone(),
                    temperature: req.temperature,
                    top_p: None,
                    max_tokens: req.max_tokens,
                    tools: req.tools.clone(),
                    session_id: None,
                    enable_thinking: true,
                    thinking_budget_tokens: Some(2048),
                    enable_prompt_caching: true,
                };
                build_anthropic_request_payload(&opts)
            }
            ProviderType::Gemini => {
                let opts = GeminiOptions {
                    model: req.model.clone(),
                    messages: req.messages.clone(),
                    temperature: req.temperature,
                    top_p: req.top_p,
                    tools: req.tools.clone(),
                    enable_thinking: true,
                    thinking_budget: Some(2048),
                    max_output_tokens: req.max_tokens,
                };
                build_gemini_request_payload(&opts)
            }
            _ => {
                let opts = OpenAIOptions {
                    model: req.model.clone(),
                    messages: req.messages.clone(),
                    temperature: req.temperature,
                    max_tokens: req.max_tokens,
                    top_p: req.top_p,
                    frequency_penalty: req.frequency_penalty,
                    presence_penalty: req.presence_penalty,
                    seed: req.seed,
                    stop: req.stop.clone(),
                    n: req.n,
                    logprobs: req.logprobs,
                    top_logprobs: req.top_logprobs,
                    stream: Some(req.stream),
                    stream_options: req.stream_options.clone(),
                    response_format: req.response_format.clone(),
                    modalities: req.modalities.clone(),
                    audio: req.audio.clone(),
                    prediction: req.prediction.clone(),
                    tools: req.tools.clone(),
                    tool_choice: req.tool_choice.clone(),
                    parallel_tool_calls: req.parallel_tool_calls,
                    custom_headers: None,
                };
                build_openai_request_payload(&opts)
            }
        };

        let start_time = Instant::now();

        // ─── 流式处理 ─────────────────────────────────────────────────────────────
        if req.stream {
            let req_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
            let model_name = req.model.clone();

            match req_builder.json(&payload).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let status_code = status.as_u16();
                        let err_text = resp.text().await.unwrap_or_default();

                        ProviderCircuitBreaker::record_failure(&prov.name);
                        if GatewayGuard::is_retryable_status(status_code) {
                            last_error = Some(GatewayError::UpstreamError {
                                status_code,
                                message: err_text,
                            });
                            continue; // 自动无感降级到下一个 Provider
                        } else {
                            return GatewayError::UpstreamError {
                                status_code,
                                message: err_text,
                            }
                            .into_response_with_lang(lang);
                        }
                    }

                    // 成功连接响应，记录一次成功
                    ProviderCircuitBreaker::record_success(&prov.name);

                    let byte_stream = resp.bytes_stream();
                    let prov_type_clone = prov_type.clone();

                    let sse_stream = async_stream::stream! {
                        let mut buffer = String::new();
                        let mut bytes_stream = byte_stream;

                        while let Some(chunk_result) = bytes_stream.next().await {
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
                                                if line.starts_with("data: ") {
                                                    let data_str = line.trim_start_matches("data: ").trim();
                                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                        if is_anthropic_stream_end(&json) {
                                                            yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
                                                            return;
                                                        }
                                                        if let Some(text_delta) = extract_anthropic_text_delta(&json) {
                                                            let chunk = build_openai_sse_chunk(&req_id, &model_name, &text_delta, None);
                                                            yield Ok::<_, Infallible>(Event::default().data(chunk.to_string()));
                                                        }
                                                    }
                                                }
                                            }
                                            ProviderType::Gemini => {
                                                if line.starts_with("data: ") {
                                                    let data_str = line.trim_start_matches("data: ").trim();
                                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                        let is_end = is_gemini_stream_end(&json);
                                                        let finish_reason = extract_gemini_finish_reason(&json);
                                                        
                                                        // 提取文本增量
                                                        if let Some(text_delta) = extract_gemini_text_delta(&json) {
                                                            let chunk = build_openai_sse_chunk(&req_id, &model_name, &text_delta, finish_reason.as_deref());
                                                            yield Ok::<_, Infallible>(Event::default().data(chunk.to_string()));
                                                        }
                                                        
                                                        // 提取 tool calls
                                                        if let Some(tool_calls) = extract_gemini_tool_calls_delta(&json) {
                                                            let tc_chunk = serde_json::json!({
                                                                "id": req_id,
                                                                "object": "chat.completion.chunk",
                                                                "created": chrono::Utc::now().timestamp(),
                                                                "model": model_name,
                                                                "choices": [{
                                                                    "index": 0,
                                                                    "delta": {
                                                                        "tool_calls": tool_calls
                                                                    },
                                                                    "finish_reason": finish_reason
                                                                }]
                                                            });
                                                            yield Ok::<_, Infallible>(Event::default().data(tc_chunk.to_string()));
                                                        }
                                                        
                                                        if is_end {
                                                            // 发送 usage (如果可用)
                                                            if let Some((prompt, completion)) = extract_gemini_usage(&json) {
                                                                let usage_chunk = serde_json::json!({
                                                                    "id": req_id,
                                                                    "object": "chat.completion.chunk",
                                                                    "created": chrono::Utc::now().timestamp(),
                                                                    "model": model_name,
                                                                    "choices": [{
                                                                        "index": 0,
                                                                        "delta": {},
                                                                        "finish_reason": finish_reason
                                                                    }],
                                                                    "usage": {
                                                                        "prompt_tokens": prompt,
                                                                        "completion_tokens": completion,
                                                                        "total_tokens": prompt + completion
                                                                    }
                                                                });
                                                                yield Ok::<_, Infallible>(Event::default().data(usage_chunk.to_string()));
                                                            }
                                                            yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                // OpenAI / Ollama / Custom 直接代理原始 SSE 行
                                                if line.starts_with("data: ") {
                                                    let data_str = line.trim_start_matches("data: ").trim();
                                                    yield Ok::<_, Infallible>(Event::default().data(data_str));
                                                    if data_str == "[DONE]" {
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
                    };

                    return Sse::new(sse_stream).into_response();
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

        // ─── 非流式处理 ────────────────────────────────────────────────────────────
        match req_builder.json(&payload).send().await {
            Ok(resp) => {
                let elapsed = start_time.elapsed().as_millis() as u64;
                let status = resp.status();

                if !status.is_success() {
                    let err_body = resp.text().await.unwrap_or_default();
                    ProviderCircuitBreaker::record_failure(&prov.name);
                    let status_code = status.as_u16();
                    if GatewayGuard::is_retryable_status(status_code) {
                        last_error = Some(GatewayError::UpstreamError {
                            status_code,
                            message: err_body,
                        });
                        continue;
                    } else {
                        return GatewayError::UpstreamError {
                            status_code,
                            message: err_body,
                        }
                        .into_response_with_lang(lang);
                    }
                }

                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        ProviderCircuitBreaker::record_success(&prov.name);
                        let mut final_response = match prov_type {
                            ProviderType::Anthropic => parse_anthropic_response(&body, elapsed),
                            ProviderType::Gemini => parse_gemini_response(&body, &req.model, elapsed),
                            _ => {
                                let choices: Vec<ChatChoice> = body
                                    .get("choices")
                                    .and_then(|c| serde_json::from_value(c.clone()).ok())
                                    .unwrap_or_default();

                                let prompt_tokens = body
                                    .pointer("/usage/prompt_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as u32;
                                let completion_tokens = body
                                    .pointer("/usage/completion_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0) as u32;

                                let tps = if elapsed > 0 {
                                    (completion_tokens as f64) / (elapsed as f64 / 1000.0)
                                } else {
                                    0.0
                                };

                                ChatCompletionResponse {
                                    id: body
                                        .get("id")
                                        .and_then(|i| i.as_str())
                                        .unwrap_or("chatcmpl-unknown")
                                        .to_string(),
                                    object: "chat.completion".to_string(),
                                    created: body
                                        .get("created")
                                        .and_then(|c| c.as_u64())
                                        .unwrap_or(chrono::Utc::now().timestamp() as u64),
                                    model: req.model.clone(),
                                    choices,
                                    usage: UsageStats {
                                        prompt_tokens,
                                        completion_tokens,
                                        total_tokens: prompt_tokens + completion_tokens,
                                    },
                                    metrics: TelemetryMetrics {
                                        latency_ms: elapsed,
                                        ttft_ms: elapsed.min(300),
                                        tps,
                                        cost_usd: 0.0,
                                    },
                                }
                            }
                        };

                        // 计算费用
                        let cost_usd = if let Some(ref item) = model_item {
                            item.calculate_cost(
                                final_response.usage.prompt_tokens as u64,
                                final_response.usage.completion_tokens as u64,
                                0,
                                0,
                            )
                        } else {
                            ((final_response.usage.prompt_tokens as f64) * 2.5
                                + (final_response.usage.completion_tokens as f64) * 10.0)
                                / 1_000_000.0
                        };
                        final_response.metrics.cost_usd = cost_usd;

                        return Json(final_response).into_response();
                    }
                    Err(e) => {
                        ProviderCircuitBreaker::record_failure(&prov.name);
                        last_error = Some(GatewayError::UpstreamError {
                            status_code: 500,
                            message: format!("解析上游响应 JSON 失败: {}", e),
                        });
                        continue;
                    }
                }
            }
            Err(e) => {
                ProviderCircuitBreaker::record_failure(&prov.name);
                last_error = Some(GatewayError::UpstreamError {
                    status_code: 500,
                    message: format!("上游 API 网络请求失败: {}", e),
                });
                continue;
            }
        }
    }

    last_error.unwrap_or_else(|| GatewayError::ProviderNotFound(req.model.clone())).into_response_with_lang(lang)
}

/// 处理 Anthropic 原生 `/v1/messages` 协议请求
pub async fn handle_anthropic_messages(
    RequestLang(lang): RequestLang,
    Json(body): Json<serde_json::Value>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("claude-3-7-sonnet");

    let prov = match GatewayDispatcher::select_provider(&cfg, model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(model.to_string()).into_response_with_lang(lang),
    };

    let target_url = format!("{}/messages", prov.base_url.trim_end_matches('/'));
    let client = SHARED_CLIENT.clone();
    let stream = body.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);

    let req_builder = client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .header("x-api-key", &prov.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("User-Agent", USER_AGENT_HEADER);

    match req_builder.json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                return GatewayError::UpstreamError {
                    status_code: status.as_u16(),
                    message: err_text,
                }
                .into_response_with_lang(lang);
            }

            if stream {
                let byte_stream = resp.bytes_stream();
                let sse_stream = async_stream::stream! {
                    let mut bytes_stream = byte_stream;
                    while let Some(chunk_result) = bytes_stream.next().await {
                        if let Ok(bytes) = chunk_result {
                            yield Ok::<_, Infallible>(Event::default().data(String::from_utf8_lossy(&bytes).to_string()));
                        }
                    }
                };
                return Sse::new(sse_stream).into_response();
            }

            if let Ok(json) = resp.json::<serde_json::Value>().await {
                return Json(json).into_response();
            }
            GatewayError::InternalError(crate::i18n::I18nManager::tr(crate::i18n::Language::from_str(lang), "error.parse_anthropic_failed", &[])).into_response_with_lang(lang)
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}

/// 处理 Gemini 原生 `/v1beta/models/*` / `/v1/models/*` 协议端点 (如 generateContent / streamGenerateContent)
pub async fn handle_gemini_native(
    RequestLang(lang): RequestLang,
    Path(path): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;
    let model = path.split(':').next().unwrap_or("gemini-2.5-flash");

    let prov = match GatewayDispatcher::select_provider(&cfg, model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(model.to_string()).into_response_with_lang(lang),
    };

    let is_stream = path.contains("streamGenerateContent");
    let target_url = format!(
        "{}/models/{}?key={}",
        prov.base_url.trim_end_matches('/'),
        path,
        prov.api_key
    );

    let client = SHARED_CLIENT.clone();

    match client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                return GatewayError::UpstreamError {
                    status_code: status.as_u16(),
                    message: err_text,
                }
                .into_response_with_lang(lang);
            }

            if is_stream {
                let byte_stream = resp.bytes_stream();
                let sse_stream = async_stream::stream! {
                    let mut bytes_stream = byte_stream;
                    while let Some(chunk_result) = bytes_stream.next().await {
                        if let Ok(bytes) = chunk_result {
                            yield Ok::<_, Infallible>(Event::default().data(String::from_utf8_lossy(&bytes).to_string()));
                        }
                    }
                };
                return Sse::new(sse_stream).into_response();
            }

            if let Ok(json) = resp.json::<serde_json::Value>().await {
                return Json(json).into_response();
            }
            GatewayError::InternalError(crate::i18n::I18nManager::tr(crate::i18n::Language::from_str(lang), "error.parse_gemini_failed", &[])).into_response_with_lang(lang)
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}

/// 处理 OpenAI /v1/responses 端点请求
pub async fn handle_responses(
    RequestLang(lang): RequestLang,
    Json(req): Json<ChatCompletionRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    let target_url = GatewayDispatcher::resolve_target_url(prov, &req.model, true);
    let (input, instructions) = ResponsesApiAdapter::convert_messages_to_responses_input(&req.messages);

    let payload = serde_json::json!({
        "model": req.model,
        "input": input,
        "instructions": instructions,
        "stream": req.stream
    });

    let client = SHARED_CLIENT.clone();

    match client
        .post(&target_url)
        .header("Authorization", format!("Bearer {}", prov.api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                return GatewayError::UpstreamError {
                    status_code: status.as_u16(),
                    message: err_text,
                }
                .into_response_with_lang(lang);
            }
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                return Json(json).into_response();
            }
            GatewayError::InternalError(crate::i18n::I18nManager::tr(crate::i18n::Language::from_str(lang), "error.parse_responses_failed", &[])).into_response_with_lang(lang)
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}
