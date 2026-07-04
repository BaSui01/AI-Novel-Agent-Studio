//! 内容精简 Agent — 对标 snow-cli/source/agents/compactAgent.ts
//!
//! 使用 basicModel 对网页/大文本进行智能提取压缩。
//! 适用于：网页预处理、信息提取、快速分析等不需要主模型的轻量任务。

use crate::agent_trait::{self, Agent, AgentConfig, AgentResult};
use gateway::types::ChatMessage;

/// 内容精简 Agent
pub struct CompactAgent {
    model: String,
    initialized: bool,
}

impl CompactAgent {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            initialized: false,
        }
    }

    /// 从网页内容中提取与用户查询相关的关键信息
    ///
    /// # Arguments
    /// * `config` - Agent 配置（含模型名）
    /// * `content` - 完整网页/文档内容
    /// * `user_query` - 用户原始查询
    /// * `url` - 网页 URL（用于上下文）
    pub async fn extract_content(
        &mut self,
        config: &AgentConfig,
        content: &str,
        user_query: &str,
        url: &str,
    ) -> AgentResult<String> {
        if !self.is_available(config) {
            // 不可用时返回原始内容
            return Ok(content.to_string());
        }

        // 如果内容过短，不需要压缩
        if content.len() < 500 {
            return Ok(content.to_string());
        }

        let prompt = format!(
            r#"You are a content extraction assistant. Extract the most relevant information from a web page based on the user's query.

User's Query: {}
Web Page URL: {}

Web Page Content:
{}

Instructions:
1. Extract ONLY the information directly relevant to the user's query
2. Preserve important details, facts, code examples, and key points
3. Remove navigation, ads, irrelevant sections, and boilerplate text
4. Organize in a clear, structured format
5. Keep technical terms and specific details intact
6. If the content is very long, focus on the most relevant sections

Provide the extracted content below:"#,
            user_query, url, content
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        match agent_trait::call_model(&self.model, &messages, Some(0.0), Some(4096)).await {
            Ok(response) => {
                if response.trim().is_empty() {
                    Ok(content.to_string())
                } else {
                    Ok(response)
                }
            }
            Err(_) => Ok(content.to_string()), // 失败回退原文
        }
    }

    /// 通用文本压缩/摘要
    pub async fn summarize_text(
        &mut self,
        config: &AgentConfig,
        text: &str,
        instruction: &str,
    ) -> AgentResult<String> {
        if !self.is_available(config) {
            return Ok(text.to_string());
        }

        if text.len() < 300 {
            return Ok(text.to_string());
        }

        let prompt = format!(
            r#"You are a text compression assistant.

Task: {}

Text to process:
{}

Output the processed result directly (no extra commentary):"#,
            instruction, text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        agent_trait::call_model(&self.model, &messages, Some(0.0), Some(4096)).await
    }
}

impl Agent for CompactAgent {
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
        "CompactAgent"
    }
}

impl Default for CompactAgent {
    fn default() -> Self {
        Self::new()
    }
}
