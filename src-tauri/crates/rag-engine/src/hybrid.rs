use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub item_type: String, // "character", "world_setting", "chapter_summary"
    pub bm25_score: f32,
    pub vector_score: f32,
    pub hybrid_score: f32,
    pub token_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub project_id: String,
    pub query: String,
    pub items: Vec<ContextItem>,
    pub total_tokens: usize,
    pub budget_tokens: usize,
}

pub struct HybridSearchEngine;

impl HybridSearchEngine {
    /// 混合评分加权算法 (Reciprocal Rank Fusion / BM25 + Vector Distance)
    pub fn calculate_hybrid_score(bm25: f32, vector_sim: f32, alpha: f32) -> f32 {
        (alpha * bm25) + ((1.0 - alpha) * vector_sim)
    }

    /// 根据 Token Budget (如 2000 Tokens) 动态裁剪组装 Context Pack
    pub fn build_context_pack(
        project_id: &str,
        query: &str,
        mut items: Vec<ContextItem>,
        budget_tokens: usize,
    ) -> ContextPack {
        // 按混合加权得分从高到低排序
        items.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap_or(std::cmp::Ordering::Equal));

        let mut packed_items = Vec::new();
        let mut accumulated_tokens = 0;

        for item in items {
            if accumulated_tokens + item.token_count <= budget_tokens {
                accumulated_tokens += item.token_count;
                packed_items.push(item);
            }
        }

        ContextPack {
            project_id: project_id.to_string(),
            query: query.to_string(),
            items: packed_items,
            total_tokens: accumulated_tokens,
            budget_tokens,
        }
    }
}
