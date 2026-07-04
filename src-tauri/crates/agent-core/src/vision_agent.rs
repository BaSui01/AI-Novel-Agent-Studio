//! 视觉 Agent — 对标 snow-cli/source/agents/visionAgent.ts
//!
//! 当主模型不支持图片识别时，使用视觉模型（visionModel）生成图片描述，
//! 将图片内容转换为文字供主模型理解。

use crate::agent_trait::{self, Agent, AgentConfig, AgentResult};
use gateway::providers::types::ImageContent;
use gateway::types::ChatMessage;

/// 视觉回退来源
#[derive(Debug, Clone, Copy)]
pub enum VisionSource {
    /// 来自用户消息
    User,
    /// 来自工具调用结果
    Tool,
}

/// 视觉 Agent
pub struct VisionAgent {
    model: String,
    initialized: bool,
}

/// 图片不可用时的提示文本
const IMAGE_UNAVAILABLE_NOTICE: &str = "Visual processing hint: The current model does not support image recognition. The image content cannot be provided.";

impl VisionAgent {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            initialized: false,
        }
    }

    /// 为不支持视觉的模型准备内容：
    /// 使用视觉模型生成图片描述，替换掉消息中的图片字段。
    ///
    /// # Arguments
    /// * `config` - Agent 配置
    /// * `content` - 原始文本内容
    /// * `images` - 图片列表
    /// * `source` - 图片来源（用户 / 工具）
    pub async fn prepare_for_non_vision_model(
        &mut self,
        config: &AgentConfig,
        content: &str,
        images: Option<&[ImageContent]>,
        source: VisionSource,
    ) -> AgentResult<(String, Option<Vec<ImageContent>>)> {
        let images = match images {
            Some(imgs) if !imgs.is_empty() => imgs,
            _ => return Ok((content.to_string(), None)),
        };

        if !self.is_available(config) {
            return Ok((self.append_notice(content), None));
        }

        match self.describe_images(config, images, source, content).await {
            Ok(Some(description)) => {
                let text = format!(
                    "{}\n\nThe visual model has generated the following description for the images:\n{}",
                    content, description
                );
                Ok((text, None))
            }
            _ => Ok((self.append_notice(content), None)),
        }
    }

    /// 调用视觉模型生成图片描述
    async fn describe_images(
        &mut self,
        _config: &AgentConfig,
        images: &[ImageContent],
        source: VisionSource,
        source_content: &str,
    ) -> AgentResult<Option<String>> {
        let source_label = match source {
            VisionSource::Tool => "tool result",
            VisionSource::User => "user message",
        };

        let context_block = if source_content.trim().is_empty() {
            String::new()
        } else {
            let truncated = if source_content.len() > 4000 {
                format!("{}...\n[truncated]", &source_content[..4000])
            } else {
                source_content.to_string()
            };
            format!(
                "\n\nSource message content from the same {} (use it to understand context):\n<source_message>\n{}\n</source_message>",
                source_label, truncated
            )
        };

        let prompt = format!(
            "The attached {} from a {}. Describe each image accurately for another AI model that cannot see images.{}",
            if images.len() == 1 { "image is" } else { "images are" },
            source_label,
            context_block,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
            images: Some(images.to_vec()),
            ..Default::default()
        }];

        match agent_trait::call_model(&self.model, &messages, Some(0.0), Some(1200)).await {
            Ok(response) if !response.is_empty() => Ok(Some(response)),
            _ => Ok(None),
        }
    }

    fn append_notice(&self, content: &str) -> String {
        if content.trim().is_empty() {
            IMAGE_UNAVAILABLE_NOTICE.to_string()
        } else {
            format!("{}\n\n{}", content, IMAGE_UNAVAILABLE_NOTICE)
        }
    }
}

impl Agent for VisionAgent {
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
        "VisionAgent"
    }
}

impl Default for VisionAgent {
    fn default() -> Self {
        Self::new()
    }
}
