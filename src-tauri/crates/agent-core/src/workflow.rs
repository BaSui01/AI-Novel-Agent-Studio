use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStep {
    PlanChapter,
    RetrieveContext,
    DraftChapter,
    ReviewChapter,
    PolishChapter,
    SummarizeChapter,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub passed: bool,
    pub ooc_warnings: Vec<String>,
    pub logic_conflicts: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub score: u8, // 1-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionState {
    pub run_id: String,
    pub current_step: WorkflowStep,
    pub current_agent: String,
    pub chapter_title: String,
    pub outline_beats: String,
    pub recalled_context: String,
    pub draft_text: String,
    pub review_report: Option<ReviewReport>,
    pub polished_text: String,
    pub summary_text: String,
    pub total_step_tokens: u64,
    pub total_step_cost_usd: f64,
    pub step_logs: Vec<String>,
}

impl WorkflowExecutionState {
    pub fn new(run_id: String, chapter_title: String) -> Self {
        Self {
            run_id,
            current_step: WorkflowStep::PlanChapter,
            current_agent: "策划 Agent (Planner)".to_string(),
            chapter_title,
            outline_beats: String::new(),
            recalled_context: String::new(),
            draft_text: String::new(),
            review_report: None,
            polished_text: String::new(),
            summary_text: String::new(),
            total_step_tokens: 0,
            total_step_cost_usd: 0.0,
            step_logs: vec!["Workflow 编排初始化完成，准备唤醒策划 Agent...".to_string()],
        }
    }
}
