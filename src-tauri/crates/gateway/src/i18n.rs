/// 支持的系统国际化语言类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    ZhCN,
    EnUS,
    JaJP,
}

impl Language {
    /// 从字符串解析语言 (支持 Accept-Language, x-language, 或配置)
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("en") {
            Language::EnUS
        } else if lower.contains("ja") {
            Language::JaJP
        } else {
            Language::ZhCN
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::EnUS => "en-US",
            Language::JaJP => "ja-JP",
        }
    }
}

/// 统一网关国际化 (i18n) 字典管理器
pub struct I18nManager;

impl I18nManager {
    /// 根据语言标识和 Key 获取翻译文本，并支持动态模板变量替换
    pub fn tr(lang: Language, key: &str, args: &[(&str, &str)]) -> String {
        let template = match (lang, key) {
            // ─── 错误响应模板 ────────────────────────────────────────────────
            (Language::ZhCN, "error.provider_not_found") => {
                "未找到可用的 AI 模型提供商: [{provider}]，请检查 config/providers.json 配置"
            }
            (Language::EnUS, "error.provider_not_found") => {
                "AI Provider not found: [{provider}]. Please check config/providers.json"
            }
            (Language::JaJP, "error.provider_not_found") => {
                "利用可能な AI プロバイダーが見つかりません: [{provider}]。config/providers.json を確認してください"
            }

            (Language::ZhCN, "error.invalid_api_key") => {
                "Provider [{provider}] 的 API Key 未配置或已失效，请前往【设置-模型配置】填写凭证"
            }
            (Language::EnUS, "error.invalid_api_key") => {
                "API Key for provider [{provider}] is missing or invalid. Please configure it in Settings."
            }
            (Language::JaJP, "error.invalid_api_key") => {
                "プロバイダー [{provider}] の API キーが未設定または無効です。[設定-モデル設定] で入力してください"
            }

            (Language::ZhCN, "error.upstream_error") => {
                "上游 API 响应异常 HTTP {status_code}: {message}"
            }
            (Language::EnUS, "error.upstream_error") => {
                "Upstream API error HTTP {status_code}: {message}"
            }
            (Language::JaJP, "error.upstream_error") => {
                "上流 API エラー HTTP {status_code}: {message}"
            }

            (Language::ZhCN, "error.network_timeout") => {
                "请求上游服务超时 (已等待 {ms} ms)，请检查网络连接或代理设置"
            }
            (Language::EnUS, "error.network_timeout") => {
                "Upstream request timed out (waited {ms} ms). Please check network/proxy settings."
            }
            (Language::JaJP, "error.network_timeout") => {
                "上流リクエストがタイムアウトしました ({ms} ms 経過)。ネットワークを確認してください"
            }

            (Language::ZhCN, "error.rate_limit_exceeded") => {
                "触发 API 频率或额度限制，请在 {secs} 秒后重试"
            }
            (Language::EnUS, "error.rate_limit_exceeded") => {
                "Rate limit exceeded. Please retry after {secs} seconds."
            }
            (Language::JaJP, "error.rate_limit_exceeded") => {
                "レート制限に達しました。{secs} 秒後に再試行してください"
            }

            (Language::ZhCN, "error.invalid_request") => "无效的请求参数: {message}",
            (Language::EnUS, "error.invalid_request") => "Invalid request parameter: {message}",
            (Language::JaJP, "error.invalid_request") => "無効なリクエストパラメータ: {message}",

            (Language::ZhCN, "error.internal_error") => "网关内部系统错误: {message}",
            (Language::EnUS, "error.internal_error") => "Gateway internal error: {message}",
            (Language::JaJP, "error.internal_error") => "ゲートウェイ内部エラー: {message}",

            (Language::ZhCN, "error.parse_gemini_failed") => "解析 Gemini 响应 JSON 失败",
            (Language::EnUS, "error.parse_gemini_failed") => "Failed to parse Gemini response JSON",
            (Language::JaJP, "error.parse_gemini_failed") => "Gemini レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_anthropic_failed") => "解析 Anthropic 响应 JSON 失败",
            (Language::EnUS, "error.parse_anthropic_failed") => "Failed to parse Anthropic response JSON",
            (Language::JaJP, "error.parse_anthropic_failed") => "Anthropic レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_responses_failed") => "解析 Responses API 响应 JSON 失败",
            (Language::EnUS, "error.parse_responses_failed") => "Failed to parse Responses API response JSON",
            (Language::JaJP, "error.parse_responses_failed") => "Responses API レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_embeddings_failed") => "解析 Embeddings 响应 JSON 失败",
            (Language::EnUS, "error.parse_embeddings_failed") => "Failed to parse Embeddings response JSON",
            (Language::JaJP, "error.parse_embeddings_failed") => "Embeddings レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_rerank_failed") => "解析 Rerank 响应 JSON 失败",
            (Language::EnUS, "error.parse_rerank_failed") => "Failed to parse Rerank response JSON",
            (Language::JaJP, "error.parse_rerank_failed") => "Rerank レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_image_failed") => "解析图片生成响应 JSON 失败",
            (Language::EnUS, "error.parse_image_failed") => "Failed to parse Image Generation response JSON",
            (Language::JaJP, "error.parse_image_failed") => "画像生成レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_image_edit_failed") => "解析图片编辑响应 JSON 失败",
            (Language::EnUS, "error.parse_image_edit_failed") => "Failed to parse Image Edit response JSON",
            (Language::JaJP, "error.parse_image_edit_failed") => "画像編集レスポンス JSON の解析に失敗しました",

            (Language::ZhCN, "error.parse_image_variation_failed") => "解析图片变体响应 JSON 失败",
            (Language::EnUS, "error.parse_image_variation_failed") => "Failed to parse Image Variation response JSON",
            (Language::JaJP, "error.parse_image_variation_failed") => "画像バリエーションレスポンス JSON の解析に失敗しました",

            // ─── 系统操作与提示模板 ───────────────────────────────────────────
            (Language::ZhCN, "msg.test_success") => {
                "模型 {model_id} 连通测试成功！响应延迟 {latency_ms}ms，HTTP {status}"
            }
            (Language::EnUS, "msg.test_success") => {
                "Model {model_id} connection test successful! Latency: {latency_ms}ms, HTTP {status}"
            }
            (Language::JaJP, "msg.test_success") => {
                "モデル {model_id} の接続テスト成功！レイテンシ: {latency_ms}ms, HTTP {status}"
            }

            (Language::ZhCN, "msg.test_failed") => {
                "模型 {model_id} 连接异常 HTTP {status}: {message}"
            }
            (Language::EnUS, "msg.test_failed") => {
                "Model {model_id} connection error HTTP {status}: {message}"
            }
            (Language::JaJP, "msg.test_failed") => {
                "モデル {model_id} 接続エラー HTTP {status}: {message}"
            }

            // 缺省直接返回 Key
            _ => key,
        };

        let mut result = template.to_string();
        for (arg_key, arg_val) in args {
            result = result.replace(&format!("{{{}}}", arg_key), arg_val);
        }
        result
    }
}
