use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub data: String,      // Base64 编码数据或 URL
    pub mime_type: String, // MIME 类型 (如 image/png, image/jpeg)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDetails {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    pub function: FunctionCallDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingInfo {
    pub thinking: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionTool {
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    pub function: FunctionDeclaration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── OpenAI Chat Completions API 扩展类型 ──

/// Structured Output — JSON Schema 模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// response_format 枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseFormat {
    Text { #[serde(rename = "type")] format_type: String },
    JsonSchema {
        #[serde(rename = "type")]
        format_type: String,
        json_schema: JsonSchemaDef,
    },
    JsonObject {
        #[serde(rename = "type")]
        format_type: String,
    },
}

/// Stream Options (include_usage / include_obfuscation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}

/// Tool Choice 策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    None(serde_json::Value),       // "none"
    Auto(serde_json::Value),       // "auto"
    Required(serde_json::Value),   // "required"
    Specific {
        #[serde(rename = "type")]
        choice_type: String,
        function: serde_json::Value,
    },
}

/// Audio 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub voice: String,
    pub format: String,
}

/// Predicted Output 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    #[serde(rename = "type")]
    pub pred_type: String, // "content"
    pub content: serde_json::Value,
}

/// 统一流式 Chunk 结构 (跨所有厂商规范归一化输出给 Frontend/Agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnifiedStreamChunk {
    Content {
        content: String,
    },
    ReasoningStarted,
    ReasoningDelta {
        delta: String,
    },
    ToolCalls {
        tool_calls: Vec<ToolCall>,
    },
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    Usage {
        usage: UsageInfo,
    },
    Done,
    Error {
        message: String,
    },
}
