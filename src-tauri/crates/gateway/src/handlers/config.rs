use axum::{extract::Json, response::IntoResponse};
use std::time::Instant;
use crate::config::GatewayConfig;
use crate::types::TestModelRequest;
use crate::{APP_NAME, APP_VERSION, SHARED_CLIENT, USER_AGENT_HEADER};
pub async fn handle_get_config() -> impl IntoResponse {
    let (cfg, _) = crate::config::get_cached().await;
    Json((*cfg).clone())
}

pub async fn handle_save_config(Json(cfg): Json<GatewayConfig>) -> impl IntoResponse {
    match crate::config::save_config("config/providers.json", cfg).await {
        Ok(_) => Json(serde_json::json!({ "status": "ok", "message": "配置保存成功" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

pub async fn handle_test_model(Json(req): Json<TestModelRequest>) -> impl IntoResponse {
    let start_time = Instant::now();
    let client = SHARED_CLIENT.clone();
    let url = format!("{}/chat/completions", req.base_url.trim_end_matches('/'));

    let mut req_builder = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Client-Name", APP_NAME)
        .header("X-Client-Version", APP_VERSION);

    if !req.api_key.is_empty() {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", req.api_key));
    }

    let payload = serde_json::json!({
        "model": req.model_id,
        "messages": [{ "role": "user", "content": "Ping" }],
        "max_tokens": 5
    });

    match req_builder.json(&payload).send().await {
        Ok(resp) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            let status = resp.status();
            if status.is_success() {
                Json(serde_json::json!({
                    "status": "ok",
                    "latency_ms": latency_ms,
                    "client_user_agent": USER_AGENT_HEADER,
                    "message": format!("模型 {} 连通测试成功！响应延迟 {}ms，HTTP {}", req.model_id, latency_ms, status)
                }))
            } else {
                let err_text = resp.text().await.unwrap_or_default();
                Json(serde_json::json!({
                    "status": "error",
                    "latency_ms": latency_ms,
                    "message": format!("模型 {} 连接异常 HTTP {}: {}", req.model_id, status, err_text)
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "latency_ms": start_time.elapsed().as_millis() as u64,
            "message": format!("网络连接错误: {}", e)
        })),
    }
}
