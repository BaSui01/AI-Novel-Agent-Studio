pub mod client;
pub mod config;
pub mod dispatcher;
pub mod embedding;
pub mod endpoint_resolver;
pub mod guards;
pub mod handlers;
pub mod i18n;
pub mod privacy_mask;
pub mod providers;
pub mod reasoning;
pub mod registry;
pub mod rerank;
pub mod responses_api;
pub mod types;

pub use client::*;
pub use config::*;
pub use dispatcher::*;
pub use embedding::*;
pub use endpoint_resolver::*;
pub use guards::*;
pub use handlers::*;
pub use i18n::*;
pub use privacy_mask::*;
pub use providers::*;
pub use reasoning::*;
pub use registry::*;
pub use rerank::*;
pub use responses_api::*;
pub use types::*;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub const APP_NAME: &str = "AI-Novel-Agent-Studio";
pub const APP_VERSION: &str = "0.4.0";
pub const USER_AGENT_HEADER: &str = "AI-Novel-Agent-Studio-Desktop/0.4.0 (Windows; Tauri 2.0; Rust Axum Gateway)";

/// 全局复用的 reqwest::Client 连接池 (支持 TCP Keep-Alive / TLS 握手复用)
pub static SHARED_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT_HEADER)
        .pool_max_idle_per_host(20)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .expect("Failed to initialize SHARED_CLIENT connection pool")
});

pub async fn start_gateway_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Config is loaded lazily via LazyLock on first access
    let _ = crate::config::get_cached().await;

    let app = Router::new()
        .route("/v1/models", get(handle_models))
        .route("/v1/models/registry", get(handle_get_model_registry).post(handle_save_model_registry))
        .route("/v1/gateway/fetch-upstream-models", post(handle_fetch_upstream_models))
        .route("/v1/gateway/test-model", post(handle_test_model))
        .route("/v1/gateway/config", get(handle_get_config).post(handle_save_config))
        .route("/v1/chat/completions", post(handle_chat_completions))
        .route("/v1/messages", post(handle_anthropic_messages))
        .route("/v1beta/models/{*path}", post(handle_gemini_native))
        .route("/v1/responses", post(handle_responses))
        .route("/v1/embeddings", post(handle_embeddings))
        .route("/v1/rerank", post(handle_rerank))
        .route("/v1/images/generations", post(handle_image_generations))
        .route("/v1/images/edits", post(handle_image_edits))
        .route("/v1/images/variations", post(handle_image_variations))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Rust AI Gateway ({}/{}) Listening on http://{}", APP_NAME, APP_VERSION, addr);

    axum::serve(listener, app).await?;
    Ok(())
}
