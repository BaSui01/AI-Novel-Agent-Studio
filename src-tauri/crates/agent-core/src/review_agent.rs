//! 代码审查 Agent — 对标 snow-cli/source/agents/reviewAgent.ts
//!
//! 使用 advancedModel 对 Git Diff 进行代码审查。
//! 检测 bug、安全问题、性能优化、代码质量等。

use crate::agent_trait::{self, Agent, AgentConfig, AgentError, AgentResult};
use gateway::types::ChatMessage;

/// 审查结果
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub summary: String,
}

/// 代码审查 Agent
pub struct ReviewAgent {
    model: String,
    initialized: bool,
}

impl ReviewAgent {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            initialized: false,
        }
    }

    /// 审查 git diff 变更
    ///
    /// # Arguments
    /// * `config` - Agent 配置（含模型名）
    /// * `git_diff` - git diff 输出
    /// * `context` - 可选的上下文说明（如项目描述）
    pub async fn review_diff(
        &mut self,
        config: &AgentConfig,
        git_diff: &str,
        context: Option<&str>,
    ) -> AgentResult<String> {
        if !self.is_available(config) {
            return Err(AgentError::NotAvailable);
        }

        if git_diff.trim().is_empty() {
            return Err(AgentError::Business("没有检测到代码变更".to_string()));
        }

        let context_section = match context {
            Some(ctx) => format!("\nProject Context: {}\n", ctx),
            None => String::new(),
        };

        let prompt = format!(
            r#"You are a senior code reviewer. Please review the following git changes and provide feedback.

**Your task:**
1. Identify potential bugs, security issues, or logic errors
2. Suggest performance optimizations
3. Point out code quality issues (readability, maintainability)
4. Check for best practices violations
5. Highlight any breaking changes or compatibility issues

**Important:**
- DO NOT modify the code yourself
- Focus on finding issues and suggesting improvements
- Be constructive and specific in your feedback
- Prioritize critical issues over minor style preferences{}

**Git Changes:**
```diff
{}
```

Please provide your review in a clear, structured format. Start with a brief overall assessment, then list specific findings with severity (🔴 Critical / 🟡 Warning / 🔵 Suggestion)."#,
            context_section, git_diff
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        agent_trait::call_model(&self.model, &messages, Some(0.3), Some(4096)).await
    }

    /// 审查代码片段（非 git diff 场景）
    pub async fn review_code(
        &mut self,
        config: &AgentConfig,
        code: &str,
        language: Option<&str>,
        focus: Option<&str>,
    ) -> AgentResult<String> {
        if !self.is_available(config) {
            return Err(AgentError::NotAvailable);
        }

        let lang_section = language.map(|l| format!("\nLanguage: {}", l)).unwrap_or_default();
        let focus_section = focus
            .map(|f| format!("\nFocus areas: {}", f))
            .unwrap_or_default();

        let prompt = format!(
            r#"You are a senior code reviewer. Please review the following code and provide feedback.

**Code to review:**{}{}

```{}
```

Please provide your review. Focus on:
1. Potential bugs or logic errors
2. Security vulnerabilities
3. Performance optimization opportunities
4. Code quality and readability
5. Best practices"#,
            lang_section, focus_section, code
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            ..Default::default()
        }];

        agent_trait::call_model(&self.model, &messages, Some(0.3), Some(4096)).await
    }
}

impl Agent for ReviewAgent {
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
        "ReviewAgent"
    }
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}
