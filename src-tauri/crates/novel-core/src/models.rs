use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub genre: String,
    pub target_audience: String,
    pub writing_style: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub sort_order: i32,
    pub summary: Option<String>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub volume_id: String,
    pub title: String,
    pub content: String,
    pub outline: Option<String>,
    pub summary: Option<String>,
    pub word_count: usize,
    pub status: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCard {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub gender: Option<String>,
    pub age: Option<String>,
    pub role_type: String,
    pub appearance: String,
    pub personality: String,
    pub goals: String,
    pub catchphrases: Vec<String>,
    pub forbidden_rules: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSetting {
    pub id: String,
    pub project_id: String,
    pub category: String,
    pub name: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub project_id: String,
    pub event_name: String,
    pub occurred_time: String,
    pub chapter_id: Option<String>,
    pub description: String,
    pub impact: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foreshadow {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub planted_chapter_id: Option<String>,
    pub resolved_chapter_id: Option<String>,
    pub status: String, // "pending", "resolved", "abandoned"
    pub importance: i32,
    pub created_at: String,
}
