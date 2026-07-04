//! 模型注册表 — 已迁移到 config.rs 的统一数据模型。
//! 保留此模块仅用于向后兼容的 re-export。

pub use crate::config::ModelInfo;

/// 兼容旧 API：从 GatewayConfig 的 model index 中查找模型
pub async fn find_model(model_id: &str) -> Option<ModelInfo> {
    let (cfg, index) = crate::config::get_cached().await;
    cfg.find_model(&index, model_id).cloned()
}
