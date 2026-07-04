use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    pub chapter_title: String,
    pub outline_beats: String,
    pub setting_knowledge: Vec<String>,
    pub previous_summary: String,
}

impl PromptContext {
    pub fn new(chapter_title: impl Into<String>) -> Self {
        Self {
            chapter_title: chapter_title.into(),
            outline_beats: String::new(),
            setting_knowledge: Vec::new(),
            previous_summary: String::new(),
        }
    }

    pub fn build_prompt(&self) -> String {
        format!(
            "章节: {}\n细纲: {}\n相关设定: {}\n前情提要: {}",
            self.chapter_title,
            self.outline_beats,
            self.setting_knowledge.join("\n"),
            self.previous_summary
        )
    }
}
