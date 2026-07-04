use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub title: String,
    pub content: String,
    pub token_count: usize,
    pub chunk_type: String, // "chapter", "character", "world_setting"
}

pub struct Chunker;

impl Chunker {
    /// 小说文本智能分块器 (包含重叠滑动窗口，防止剧情断层)
    pub fn split_text(
        doc_id: &str,
        title: &str,
        text: &str,
        chunk_size: usize,
        overlap: usize,
        chunk_type: &str,
    ) -> Vec<TextChunk> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_text = String::new();
        let mut idx = 0;

        for p in paragraphs {
            let p_trimmed = p.trim();
            if p_trimmed.is_empty() {
                continue;
            }

            if current_text.len() + p_trimmed.len() > chunk_size && !current_text.is_empty() {
                let token_count = current_text.len() / 4;
                chunks.push(TextChunk {
                    chunk_id: format!("{}_chunk_{}", doc_id, idx),
                    document_id: doc_id.to_string(),
                    title: title.to_string(),
                    content: current_text.clone(),
                    token_count,
                    chunk_type: chunk_type.to_string(),
                });
                idx += 1;

                // 保留尾部 overlap 长度作为滑动重叠窗口
                if current_text.len() > overlap {
                    current_text = current_text[current_text.len() - overlap..].to_string();
                } else {
                    current_text.clear();
                }
            }

            if !current_text.is_empty() {
                current_text.push_str("\n\n");
            }
            current_text.push_str(p_trimmed);
        }

        if !current_text.is_empty() {
            let token_count = current_text.len() / 4;
            chunks.push(TextChunk {
                chunk_id: format!("{}_chunk_{}", doc_id, idx),
                document_id: doc_id.to_string(),
                title: title.to_string(),
                content: current_text,
                token_count,
                chunk_type: chunk_type.to_string(),
            });
        }

        chunks
    }
}
