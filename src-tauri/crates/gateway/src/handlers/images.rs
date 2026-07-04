use axum::{extract::Json, response::IntoResponse};

use crate::types::ProviderType;
use crate::dispatcher::GatewayDispatcher;
use crate::endpoint_resolver::{normalize_base_url, strip_known_endpoint_suffix};
use crate::i18n::{I18nManager, Language};
use crate::providers::{
    build_gemini_image_gen_payload, build_openai_image_edit_payload, build_openai_image_gen_payload,
    build_openai_image_variation_payload, normalize_gemini_image_response, ImageEditRequest,
    ImageGenRequest, ImageVariationRequest,
};
use crate::types::{GatewayError, RequestLang};
use crate::SHARED_CLIENT;

fn tr(lang: &str, key: &str) -> String {
    I18nManager::tr(Language::from_str(lang), key, &[])
}

/// ─── POST /v1/images/generations ─────────────────────────────────────────────
pub async fn handle_image_generations(
    RequestLang(lang): RequestLang,
    Json(req): Json<ImageGenRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    let prov_type = GatewayDispatcher::detect_provider_type(prov);
    let client = SHARED_CLIENT.clone();

    let (target_url, payload) = match prov_type {
        ProviderType::Gemini => {
            let base = prov.base_url.trim_end_matches('/');
            let url = format!("{}/models/{}:generateContent?key={}", base, req.model, prov.api_key);
            (url, build_gemini_image_gen_payload(&req))
        }
        _ => {
            let base = normalize_base_url(&prov.base_url);
            let url = format!("{}/images/generations", strip_known_endpoint_suffix(&base));
            let p = build_openai_image_gen_payload(&req);
            // 注入 API Key 已在 Header 中处理，不放 payload
            (url, p)
        }
    };

    let mut req_builder = client
        .post(&target_url)
        .header("Content-Type", "application/json");

    if !prov.api_key.is_empty() && prov_type != ProviderType::Gemini {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", prov.api_key));
    }

    match req_builder.json(&payload).send().await {
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
            match resp.json::<serde_json::Value>().await {
                Ok(body) => {
                    let normalized = if prov_type == ProviderType::Gemini {
                        normalize_gemini_image_response(&body)
                    } else {
                        body
                    };
                    Json(normalized).into_response()
                }
                Err(_) => {
                    GatewayError::InternalError(tr(lang, "error.parse_image_failed")).into_response_with_lang(lang)
                }
            }
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}

/// ─── POST /v1/images/edits ────────────────────────────────────────────────────
pub async fn handle_image_edits(
    RequestLang(lang): RequestLang,
    Json(req): Json<ImageEditRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    let base = normalize_base_url(&prov.base_url);
    let target_url = format!("{}/images/edits", strip_known_endpoint_suffix(&base));
    let payload = build_openai_image_edit_payload(&req);
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
            match resp.json::<serde_json::Value>().await {
                Ok(body) => Json(body).into_response(),
                Err(_) => GatewayError::InternalError(tr(lang, "error.parse_image_edit_failed")).into_response_with_lang(lang),
            }
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}

/// ─── POST /v1/images/variations ──────────────────────────────────────────────
pub async fn handle_image_variations(
    RequestLang(lang): RequestLang,
    Json(req): Json<ImageVariationRequest>,
) -> axum::response::Response {
    let (cfg, _) = crate::config::get_cached().await;

    let prov = match GatewayDispatcher::select_provider(&cfg, &req.model) {
        Some(p) => p,
        None => return GatewayError::ProviderNotFound(req.model.clone()).into_response_with_lang(lang),
    };

    let base = normalize_base_url(&prov.base_url);
    let target_url = format!("{}/images/variations", strip_known_endpoint_suffix(&base));
    let payload = build_openai_image_variation_payload(&req);
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
            match resp.json::<serde_json::Value>().await {
                Ok(body) => Json(body).into_response(),
                Err(_) => GatewayError::InternalError(tr(lang, "error.parse_image_variation_failed")).into_response_with_lang(lang),
            }
        }
        Err(e) => GatewayError::UpstreamError {
            status_code: 500,
            message: e.to_string(),
        }
        .into_response_with_lang(lang),
    }
}
