use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

/// 模型元信息（含定价、能力标签、上下文窗口等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub api_model_name: String,
    #[serde(default = "default_endpoint_type")]
    pub endpoint_type: String, // "chat_completions" | "responses"
    #[serde(default = "default_context_window")]
    pub context_window: u32,
    #[serde(default = "default_max_output")]
    pub max_output_tokens: u32,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default = "default_true")]
    pub supports_tools: bool,
    #[serde(default = "default_true")]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub input_price_per_1m: f64,
    #[serde(default)]
    pub output_price_per_1m: f64,
    #[serde(default)]
    pub cached_input_price_per_1m: f64,
    #[serde(default)]
    pub reasoning_price_per_1m: f64,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_custom: bool,
}

fn default_endpoint_type() -> String {
    "chat_completions".to_string()
}
fn default_context_window() -> u32 {
    128000
}
fn default_max_output() -> u32 {
    8192
}
fn default_true() -> bool {
    true
}
fn default_currency() -> String {
    "USD".to_string()
}

impl ModelInfo {
    /// 计算单次请求费用 (USD)
    pub fn calculate_cost(
        &self,
        prompt_tokens: u64,
        completion_tokens: u64,
        _prompt_cache_hit_tokens: u64,
        _prompt_cache_write_tokens: u64,
    ) -> f64 {
        let prompt_cost = (prompt_tokens as f64) * self.input_price_per_1m / 1_000_000.0;
        let completion_cost = (completion_tokens as f64) * self.output_price_per_1m / 1_000_000.0;
        prompt_cost + completion_cost
    }

    /// 为动态发现的模型创建"乐观默认"条目
    pub fn create_default(id: &str) -> Self {
        ModelInfo {
            id: id.to_string(),
            display_name: id.to_string(),
            api_model_name: id.to_string(),
            endpoint_type: "chat_completions".to_string(),
            context_window: 128000,
            max_output_tokens: 8192,
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            supports_reasoning: false,
            input_price_per_1m: 0.0,
            output_price_per_1m: 0.0,
            cached_input_price_per_1m: 0.0,
            reasoning_price_per_1m: 0.0,
            currency: "USD".to_string(),
            enabled: true,
            is_custom: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderItem {
    pub name: String,
    pub display_name: String,
    pub base_url: String,
    #[serde(default)]
    pub chat_completions_url: Option<String>,
    #[serde(default)]
    pub responses_url: Option<String>,
    #[serde(default)]
    pub openai_compat_url: Option<String>,
    pub api_key: String,
    pub is_enabled: bool,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub active_provider: String,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub providers: Vec<ProviderItem>,
}

/// 模型快速索引: model_id → (provider_index, model_index)
#[derive(Debug, Clone)]
pub struct ModelIndex {
    /// model_id (lowercased) → (provider_index, model_index)
    pub map: HashMap<String, (usize, usize)>,
}

impl GatewayConfig {
    /// 构建 model_id → (provider_idx, model_idx) 的 O(1) 索引
    pub fn build_index(&self) -> ModelIndex {
        let mut map = HashMap::new();
        for (pi, prov) in self.providers.iter().enumerate() {
            for (mi, model) in prov.models.iter().enumerate() {
                map.insert(model.id.to_lowercase(), (pi, mi));
            }
        }
        ModelIndex { map }
    }

    /// 通过 model_id 查找 ModelInfo
    pub fn find_model(&self, index: &ModelIndex, model_id: &str) -> Option<&ModelInfo> {
        let (pi, mi) = index.map.get(&model_id.to_lowercase())?;
        self.providers.get(*pi)?.models.get(*mi)
    }

    /// 通过 model_id 查找所属 ProviderItem
    pub fn find_provider_for_model<'a>(
        &'a self,
        index: &ModelIndex,
        model_id: &str,
    ) -> Option<&'a ProviderItem> {
        let (pi, _) = index.map.get(&model_id.to_lowercase())?;
        self.providers.get(*pi)
    }
}

/// 全局配置缓存：Arc<GatewayConfig> + ModelIndex
static GLOBAL_CONFIG: LazyLock<RwLock<(Arc<GatewayConfig>, ModelIndex)>> =
    LazyLock::new(|| {
        let cfg = GatewayConfig::load_or_create_default("config/providers.json");
        let index = cfg.build_index();
        RwLock::new((Arc::new(cfg), index))
    });

/// 零磁盘 I/O 获取当前有效的网关配置和模型索引
pub async fn get_cached() -> (Arc<GatewayConfig>, ModelIndex) {
    GLOBAL_CONFIG.read().await.clone()
}

/// 原子化保存配置文件并同步更新缓存
pub async fn save_config(
    config_path: &str,
    new_config: GatewayConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string_pretty(&new_config)?;
    // 原子写入：先写临时文件，再 rename
    let temp_path = format!("{}.tmp", config_path);
    fs::write(&temp_path, &json)?;
    fs::rename(&temp_path, config_path)?;

    let index = new_config.build_index();
    let mut cache = GLOBAL_CONFIG.write().await;
    *cache = (Arc::new(new_config), index);
    Ok(())
}

/// 设置配置目录（用于启动时指定配置路径）
static CONFIG_PATH: LazyLock<std::sync::Mutex<String>> = LazyLock::new(|| {
    std::sync::Mutex::new("config/providers.json".to_string())
});

pub fn set_config_dir(dir: &str) {
    let mut path = CONFIG_PATH.lock().unwrap();
    *path = format!("{}/providers.json", dir.trim_end_matches('/'));
}

impl GatewayConfig {
    /// 加载网关 Provider 配置。若文件不存在，创建干净的空配置 (无任何硬编码模型)
    pub fn load_or_create_default<P: AsRef<Path>>(config_path: P) -> Self {
        let path = config_path.as_ref();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str::<GatewayConfig>(&content) {
                    return cfg;
                }
            }
        }

        // 干净配置，不硬编码任何 API Key，但提供默认模型元信息
        let default_cfg = GatewayConfig {
            active_provider: "openai".to_string(),
            fallback_chain: vec![
                "openai".to_string(),
                "claude".to_string(),
                "gemini".to_string(),
                "ollama".to_string(),
            ],
            providers: vec![
                ProviderItem {
                    name: "openai".to_string(),
                    display_name: "OpenAI".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    chat_completions_url: Some("https://api.openai.com/v1/chat/completions".to_string()),
                    responses_url: Some("https://api.openai.com/v1/responses".to_string()),
                    openai_compat_url: None,
                    api_key: String::new(),
                    is_enabled: true,
                    models: vec![
                        ModelInfo::create_default("gpt-4o"),
                        ModelInfo::create_default("gpt-4o-mini"),
                    ],
                },
                ProviderItem {
                    name: "claude".to_string(),
                    display_name: "Anthropic Claude".to_string(),
                    base_url: "https://api.anthropic.com/v1".to_string(),
                    chat_completions_url: None,
                    responses_url: None,
                    openai_compat_url: None,
                    api_key: String::new(),
                    is_enabled: true,
                    models: vec![
                        ModelInfo::create_default("claude-3-5-sonnet-20241022"),
                        ModelInfo::create_default("claude-3-5-haiku-20241022"),
                    ],
                },
                ProviderItem {
                    name: "gemini".to_string(),
                    display_name: "Google Gemini".to_string(),
                    base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                    chat_completions_url: None,
                    responses_url: None,
                    openai_compat_url: Some(
                        "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                    ),
                    api_key: String::new(),
                    is_enabled: true,
                    models: vec![
                        ModelInfo::create_default("gemini-2.0-flash"),
                        ModelInfo::create_default("gemini-1.5-pro"),
                    ],
                },
                ProviderItem {
                    name: "ollama".to_string(),
                    display_name: "Ollama Local".to_string(),
                    base_url: "http://127.0.0.1:11434".to_string(),
                    chat_completions_url: None,
                    responses_url: None,
                    openai_compat_url: None,
                    api_key: String::new(),
                    is_enabled: true,
                    models: vec![
                        ModelInfo::create_default("qwen2.5:32b"),
                        ModelInfo::create_default("deepseek-r1:14b"),
                    ],
                },
            ],
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write(path, json);
        }

        default_cfg
    }
}
