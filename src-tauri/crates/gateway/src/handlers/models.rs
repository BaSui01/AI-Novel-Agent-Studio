use axum::{extract::Json, response::IntoResponse};
use crate::config::ModelInfo;
use crate::types::FetchUpstreamRequest;
use crate::{APP_NAME, APP_VERSION, SHARED_CLIENT};

pub async fn handle_models() -> impl IntoResponse {
    let (cfg, _) = crate::config::get_cached().await;
    let mut model_list = Vec::new();

    for prov in &cfg.providers {
        for m in &prov.models {
            if m.enabled {
                model_list.push(serde_json::json!({
                    "id": m.id,
                    "object": "model",
                    "owned_by": prov.name
                }));
            }
        }
    }

    Json(serde_json::json!({
        "object": "list",
        "data": model_list
    }))
}

pub async fn handle_get_model_registry() -> impl IntoResponse {
    let (cfg, _) = crate::config::get_cached().await;
    let all_models: Vec<ModelInfo> = cfg.providers.iter().flat_map(|p| p.models.iter().cloned()).collect();
    Json(all_models)
}

pub async fn handle_save_model_registry(Json(items): Json<Vec<ModelInfo>>) -> impl IntoResponse {
    // Update models in config: rebuild provider models from flat list
    // For now, save the full config with updated model info
    let (cfg, _) = crate::config::get_cached().await;
    let mut new_cfg = (*cfg).clone();

    // Merge incoming model info into existing providers
    for item in &items {
        for prov in &mut new_cfg.providers {
            if let Some(existing) = prov.models.iter_mut().find(|m| m.id == item.id) {
                *existing = item.clone();
            }
        }
    }

    match crate::config::save_config("config/providers.json", new_cfg).await {
        Ok(_) => Json(serde_json::json!({ "status": "ok", "message": "模型注册表保存成功" })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

pub async fn handle_fetch_upstream_models(Json(req): Json<FetchUpstreamRequest>) -> impl IntoResponse {
    let client = SHARED_CLIENT.clone();
    let url = format!("{}/models", req.base_url.trim_end_matches('/'));

    let mut request_builder = client
        .get(&url)
        .header("X-Client-Name", APP_NAME)
        .header("X-Client-Version", APP_VERSION);

    if !req.api_key.is_empty() {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", req.api_key));
    }

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let mut models = Vec::new();
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        for item in data {
                            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                models.push(id.to_string());
                            }
                        }
                    }
                    return Json(serde_json::json!({
                        "status": "ok",
                        "models": models
                    }));
                }
            }
            Json(serde_json::json!({
                "status": "error",
                "message": format!("上游 API 返回 HTTP {}，未找到标准 data:[{{id}}] 结构", status)
            }))
        }
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
