/// API 端点自动解析器 (对标 snow-cli/source/api/endpointResolver.ts)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiEndpointKind {
    Chat,
    Responses,
    Models,
    AnthropicMessages,
    GeminiStreamGenerateContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseUrlMode {
    #[default]
    Auto,
    Endpoint,
}

pub fn normalize_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

pub fn strip_known_endpoint_suffix(url: &str) -> String {
    let normalized = normalize_base_url(url);
    let suffixes = [
        "/chat/completions",
        "/responses",
        "/models",
        "/messages",
        "/v1",
    ];

    let mut current = normalized;
    for suffix in suffixes {
        if current.ends_with(suffix) {
            current = current[..current.len() - suffix.len()].trim_end_matches('/').to_string();
        }
    }
    current
}

pub fn resolve_api_endpoint(
    base_url: &str,
    kind: ApiEndpointKind,
    mode: BaseUrlMode,
    model_name: Option<&str>,
    api_key: Option<&str>,
    is_stream: bool,
) -> String {
    let normalized = normalize_base_url(base_url);

    if mode == BaseUrlMode::Endpoint {
        return normalized;
    }

    match kind {
        ApiEndpointKind::Chat => {
            if normalized.ends_with("/chat/completions") {
                normalized
            } else {
                format!("{}/chat/completions", normalized)
            }
        }
        ApiEndpointKind::Responses => {
            if normalized.ends_with("/responses") {
                normalized
            } else {
                format!("{}/responses", normalized)
            }
        }
        ApiEndpointKind::Models => {
            if normalized.ends_with("/models") {
                normalized
            } else {
                format!("{}/models", normalized)
            }
        }
        ApiEndpointKind::AnthropicMessages => {
            if normalized.ends_with("/messages") {
                normalized
            } else {
                format!("{}/messages", normalized)
            }
        }
        ApiEndpointKind::GeminiStreamGenerateContent => {
            if normalized.contains("generativelanguage.googleapis.com") || normalized.is_empty() {
                let model = model_name.unwrap_or("gemini-2.5-flash");
                let key = api_key.unwrap_or("");
                let action = if is_stream {
                    "streamGenerateContent?alt=sse"
                } else {
                    "generateContent"
                };
                format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:{}?key={}",
                    model, action, key
                )
            } else if normalized.ends_with("generateContent") || normalized.contains("streamGenerateContent") {
                normalized
            } else {
                let action = if is_stream {
                    "streamGenerateContent?alt=sse"
                } else {
                    "generateContent"
                };
                let model = model_name.unwrap_or("gemini-2.5-flash");
                format!("{}/models/{}:{}", normalized, model, action)
            }
        }
    }
}
