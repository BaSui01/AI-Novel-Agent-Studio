use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: serde_json::Value, // String or Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingDataItem {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingDataItem>,
    pub model: String,
    pub usage: crate::UsageStats,
}

/// 简单 LRU 缓存结构
struct EmbeddingLruCache {
    map: HashMap<String, serde_json::Value>,
    order: VecDeque<String>,
    max_size: usize,
}

impl EmbeddingLruCache {
    fn new(max_size: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            max_size,
        }
    }

    fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.map.get(key)
    }

    fn insert(&mut self, key: String, value: serde_json::Value) {
        if self.map.len() >= self.max_size {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }
}

/// 内存级 Embeddings LRU 缓存存储器 (实现 0ms 提速，最多 1000 条)
static EMBEDDING_CACHE: LazyLock<RwLock<EmbeddingLruCache>> =
    LazyLock::new(|| RwLock::new(EmbeddingLruCache::new(1000)));

pub struct EmbeddingCacheStore;

impl EmbeddingCacheStore {
    /// 计算 (model + input) 的 SHA-256 唯一指纹哈希
    pub fn compute_cache_key(model: &str, input: &serde_json::Value) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", model, input).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 尝试从缓存中提取响应
    pub fn get(model: &str, input: &serde_json::Value) -> Option<serde_json::Value> {
        let key = Self::compute_cache_key(model, input);
        if let Ok(guard) = EMBEDDING_CACHE.try_read() {
            return guard.get(&key).cloned();
        }
        None
    }

    /// 写入缓存
    pub fn set(model: &str, input: &serde_json::Value, response: serde_json::Value) {
        let key = Self::compute_cache_key(model, input);
        if let Ok(mut guard) = EMBEDDING_CACHE.try_write() {
            guard.insert(key, response);
        }
    }
}

/// Embedding 向量提取组件 (用于小说设定集与知识库 RAG 相似度搜索)
pub struct EmbeddingAdapter;

impl EmbeddingAdapter {
    pub fn build_openai_embedding_payload(model: &str, input: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "model": model,
            "input": input
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_cache_hit() {
        let model = "text-embedding-3-small";
        let input = serde_json::json!("小说主角叫李林，天赋极高。");
        let mock_resp = serde_json::json!({
            "object": "list",
            "data": [{"embedding": [0.1, 0.2, 0.3], "index": 0}],
            "model": model
        });

        EmbeddingCacheStore::set(model, &input, mock_resp.clone());
        let cached = EmbeddingCacheStore::get(model, &input);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()["model"], model);
    }
}
