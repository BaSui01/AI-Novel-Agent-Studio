use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankResultItem {
    pub index: usize,
    pub relevance_score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RerankResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<RerankResultItem>,
}

/// 重排序 (Rerank) API 适配器 (用于小说 RAG 设定精细排序)
pub struct RerankAdapter;

impl RerankAdapter {
    pub fn build_jina_cohere_rerank_payload(
        model: &str,
        query: &str,
        documents: &[String],
        top_n: Option<usize>,
    ) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "model": model,
            "query": query,
            "documents": documents
        });

        if let Some(n) = top_n {
            payload["top_n"] = serde_json::json!(n);
        }

        payload
    }
}
