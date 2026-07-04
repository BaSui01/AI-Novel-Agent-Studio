//! 终端输出摘要 Agent — 对标 snow-cli/source/agents/bashOutputSummaryAgent.ts
//!
//! 使用 basicModel 将嘈杂的终端命令输出压缩为有用的结构化信息。
//! 遵循 "错误优先" 原则。

use crate::agent_trait::{self, Agent, AgentConfig, AgentResult};
use gateway::types::ChatMessage;

/// 命令执行结果
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 终端输出摘要 Agent
pub struct BashOutputSummaryAgent {
    model: String,
    initialized: bool,
}

impl BashOutputSummaryAgent {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            initialized: false,
        }
    }

    /// 压缩终端命令输出
    ///
    /// 如果 Agent 不可用，直接返回原始输出（保留 stdout/stderr）。
    pub async fn summarize(
        &mut self,
        config: &AgentConfig,
        result: &CommandResult,
    ) -> AgentResult<CommandResult> {
        if !self.is_available(config) {
            return Ok(result.clone());
        }

        let prompt = format!(
            r#"You are a terminal output compression assistant. Compress noisy command output into useful, actionable information for another AI agent.

Requirements:
1) Keep factual correctness. Do not invent outputs.
2) Error-first policy: always report errors before warnings, even if warning volume is much higher.
3) If any errors exist, list all unique errors with exact lines/snippets and likely impact first.
4) Prioritize actionable next steps, key artifacts/paths, and final status after errors/warnings.
5) Remove repetitive logs, progress bars, and low-value noise.
6) Keep language concise and structured.
7) Preserve important command snippets and exact error lines when needed.
8) Output plain text only.

Command: {}
Exit code: {}

STDOUT:
{}

STDERR:
{}

Now produce the compressed terminal result:"#,
            result.command, result.exit_code, result.stdout, result.stderr
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        match agent_trait::call_model(&self.model, &messages, Some(0.0), Some(1200)).await {
            Ok(summary) if !summary.is_empty() => Ok(CommandResult {
                stdout: summary,
                stderr: String::new(),
                ..result.clone()
            }),
            _ => Ok(result.clone()), // 失败回退
        }
    }
}

impl Agent for BashOutputSummaryAgent {
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
        "BashOutputSummaryAgent"
    }
}

impl Default for BashOutputSummaryAgent {
    fn default() -> Self {
        Self::new()
    }
}
