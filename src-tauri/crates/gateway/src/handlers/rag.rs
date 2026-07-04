use axum::{extract::Json, response::IntoResponse};
use crate::dispatcher::GatewayDispatcher;
use crate::embedding::{EmbeddingAdapter, EmbeddingCacheStore, EmbeddingRequest};
use crate::rerank::{RerankAdapter, RerankRequest};
use crate::types::{GatewayError, RequestLang};
use crate::SHARED_CLIENT;

pub async fn handle_embeddings(
    RequestLang(lang): RequestLang,
    Json(req): Json<EmbeddingRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    if let Some(cached) = EmbeddingCacheStore::get(&req.model, &req.input) {
        return Json(cached).into_response();
    }

    let target_url = format!("{}/embeddings", prov.base_url.trim_end_matches('/'));
    let payload = EmbeddingAdapter::build_openai_embedding_payload(&req.model, &req.input);

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
                EmbeddingCacheStore::set(&req.model, &req.input, json.clone());
                return Json(json).into_response();
            }
            GatewayError::InternalError(crate::i18n::I18nManager::tr(crate::i18n::Language::from_str(lang), "error.parse_embeddings_failed", &[])).into_response_with_lang(lang)
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}

pub async fn handle_rerank(
    RequestLang(lang): RequestLang,
    Json(req): Json<RerankRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    let target_url = format!("{}/rerank", prov.base_url.trim_end_matches('/'));
    let payload = RerankAdapter::build_jina_cohere_rerank_payload(&req.model, &req.query, &req.documents, req.top_n);

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
            GatewayError::InternalError(crate::i18n::I18nManager::tr(crate::i18n::Language::from_str(lang), "error.parse_rerank_failed", &[])).into_response_with_lang(lang)
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}
