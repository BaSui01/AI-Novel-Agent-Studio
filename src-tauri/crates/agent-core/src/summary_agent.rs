//! 对话摘要 Agent — 对标 snow-cli/source/agents/summaryAgent.ts
//!
//! 使用 basicModel 生成对话标题（≤50 字符）和摘要（≤150 字符）。
//! 只会在第一次完整对话交换后运行一次。

use crate::agent_trait::{self, Agent, AgentConfig, AgentResult, extract_json_from_response, truncate_string};
use gateway::types::ChatMessage;

/// 生成的摘要
#[derive(Debug, Clone)]
pub struct ConversationSummary {
    pub title: String,
    pub summary: String,
}

/// 对话摘要 Agent
pub struct SummaryAgent {
    model: String,
    initialized: bool,
}

impl SummaryAgent {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            initialized: false,
        }
    }

    /// 根据用户第一条消息和 AI 第一条回复生成标题+摘要
    pub async fn generate_summary(
        &mut self,
        config: &AgentConfig,
        user_message: &str,
        assistant_message: &str,
    ) -> AgentResult<ConversationSummary> {
        if !self.is_available(config) {
            return Ok(self.fallback_summary(user_message));
        }

        let prompt = format!(
            r#"You are a conversation summarization assistant. Based on the first exchange between the user and AI assistant below, generate a concise title and summary.
IMPORTANT: Generate the title and summary in the SAME LANGUAGE as the user's message.

User message: {}
AI assistant reply: {}

Requirements:
1. Generate a short title (max 50 characters) that captures the conversation topic
2. Generate a summary (max 150 characters) that briefly describes the core content
3. Title should be concise and clear, avoid complete sentences  
4. Summary should contain key information while staying brief
5. Use the SAME LANGUAGE as the user's message

Output in the following JSON format (JSON only, no other content):
{{ "title": "Conversation title", "summary": "Conversation summary" }}"#,
            user_message, assistant_message
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        match agent_trait::call_model(&self.model, &messages, Some(0.0), Some(1200)).await {
            Ok(response) => {
                if let Some(json) = extract_json_from_response(&response) {
                    let title = json["title"]
                        .as_str()
                        .map(|s| truncate_string(s, 50))
                        .unwrap_or_else(|| truncate_string(user_message, 50));

                    let summary = json["summary"]
                        .as_str()
                        .map(|s| truncate_string(s, 150))
                        .unwrap_or_else(|| truncate_string(user_message, 150));

                    return Ok(ConversationSummary { title, summary });
                }
                // JSON 解析失败，回退
                Ok(self.fallback_summary(user_message))
            }
            Err(_) => Ok(self.fallback_summary(user_message)),
        }
    }

    /// 回退摘要：直接截断用户消息
    fn fallback_summary(&self, user_message: &str) -> ConversationSummary {
        let cleaned = user_message
            .replace('\n', " ")
            .replace('\r', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        ConversationSummary {
            title: truncate_string(&cleaned, 50),
            summary: truncate_string(&cleaned, 150),
        }
    }
}

impl Agent for SummaryAgent {
    fn initialize(&mut self, config: &AgentConfig) -> bool {
        if let Some(ref model) = config.model {
            if !model.is_empty() {
                self.model = model.clone();
                self.initialized = true;
                return true;
            }
        }
        false
    }

    fn clear_cache(&mut self) {
        self.initialized = false;
        self.model.clear();
    }

    fn is_available(&mut self, config: &AgentConfig) -> bool {
        if !self.initialized {
            return self.initialize(config);
        }
        true
    }

    fn name(&self) -> &'static str {
        "SummaryAgent"
    }
}

impl Default for SummaryAgent {
    fn default() -> Self {
        Self::new()
    }
}
