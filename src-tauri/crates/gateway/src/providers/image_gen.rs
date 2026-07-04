use serde::{Deserialize, Serialize};

/// ─── 文生图请求 (/v1/images/generations) ──────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>, // "1024x1024" | "1536x1024" | "1024x1536" | "auto"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>, // "auto" | "high" | "medium" | "low"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>, // "vivid" | "natural" (DALL·E 3 only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>, // "b64_json" | "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>, // "png" | "jpeg" | "webp"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u8>, // 0-100, webp/jpeg only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>, // "transparent" | "opaque" | "auto"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>, // "auto" | "low"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// ─── 图片编辑请求 (/v1/images/edits) ─────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEditRequest {
    pub model: String,
    pub prompt: String,
    /// Base64 编码的原始图片 (PNG, RGBA支持透明度用于 mask 区域)
    pub image: Vec<ImageInputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>, // Base64 PNG mask (白色=编辑区域, 黑色=保留)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>, // "b64_json" | "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// 图片输入项 (支持单张 Base64 / URL 两种形式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInputItem {
    #[serde(rename = "type")]
    pub input_type: String, // "base64" | "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>, // "auto" | "low" | "high"
}

/// ─── 图片变体请求 (/v1/images/variations) ────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariationRequest {
    pub model: String,
    pub image: String, // Base64 PNG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>, // "b64_json" | "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// ─── 统一图片响应结构 ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenResponse {
    pub created: u64,
    pub data: Vec<GeneratedImageData>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Payload Builders
// ─────────────────────────────────────────────────────────────────────────────

/// 构建 OpenAI gpt-image-2 / dall-e-3 文生图 Payload
pub fn build_openai_image_gen_payload(req: &ImageGenRequest) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": if req.model.is_empty() { "gpt-image-2" } else { &req.model },
        "prompt": req.prompt,
        "n": req.n.unwrap_or(1),
        "response_format": req.response_format.as_deref().unwrap_or("b64_json")
    });

    if let Some(ref s) = req.size { payload["size"] = serde_json::json!(s); }
    if let Some(ref q) = req.quality { payload["quality"] = serde_json::json!(q); }
    if let Some(ref s) = req.style { payload["style"] = serde_json::json!(s); }
    if let Some(ref f) = req.output_format { payload["output_format"] = serde_json::json!(f); }
    if let Some(c) = req.output_compression { payload["output_compression"] = serde_json::json!(c); }
    if let Some(ref b) = req.background { payload["background"] = serde_json::json!(b); }
    if let Some(ref m) = req.moderation { payload["moderation"] = serde_json::json!(m); }
    if let Some(ref u) = req.user { payload["user"] = serde_json::json!(u); }

    payload
}

/// 构建 OpenAI 图片编辑 Payload (gpt-image-2, DALL·E 2 compatible)
pub fn build_openai_image_edit_payload(req: &ImageEditRequest) -> serde_json::Value {
    // 统一转换为 images 数组格式 (gpt-image-2 多图支持)
    let images: Vec<serde_json::Value> = req.image.iter().map(|img| {
        match img.input_type.as_str() {
            "url" => serde_json::json!({ "type": "url", "url": img.url }),
            _ => serde_json::json!({ "type": "base64", "b64_json": img.b64_json }),
        }
    }).collect();

    let mut payload = serde_json::json!({
        "model": req.model,
        "prompt": req.prompt,
        "image": images,
        "n": req.n.unwrap_or(1),
        "response_format": req.response_format.as_deref().unwrap_or("b64_json")
    });

    if let Some(ref mask) = req.mask { payload["mask"] = serde_json::json!(mask); }
    if let Some(ref s) = req.size { payload["size"] = serde_json::json!(s); }
    if let Some(ref q) = req.quality { payload["quality"] = serde_json::json!(q); }
    if let Some(ref f) = req.output_format { payload["output_format"] = serde_json::json!(f); }
    if let Some(c) = req.output_compression { payload["output_compression"] = serde_json::json!(c); }
    if let Some(ref u) = req.user { payload["user"] = serde_json::json!(u); }

    payload
}

/// 构建 OpenAI 图片变体 Payload (DALL·E 2 compatible)
pub fn build_openai_image_variation_payload(req: &ImageVariationRequest) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": req.model,
        "image": req.image,
        "n": req.n.unwrap_or(1),
        "response_format": req.response_format.as_deref().unwrap_or("b64_json")
    });

    if let Some(ref s) = req.size { payload["size"] = serde_json::json!(s); }
    if let Some(ref u) = req.user { payload["user"] = serde_json::json!(u); }

    payload
}

/// 构建 Gemini generateContent 文生图 Payload
pub fn build_gemini_image_gen_payload(req: &ImageGenRequest) -> serde_json::Value {
    let mut gen_config = serde_json::json!({
        "responseModalities": ["TEXT", "IMAGE"]
    });

    if let Some(ref s) = req.size {
        if s.contains(':') {
            // 宽高比格式 (如 "16:9", "4:3", "1:1")
            gen_config["aspectRatio"] = serde_json::json!(s);
        } else {
            let parts: Vec<&str> = s.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    gen_config["imageConfig"] = serde_json::json!({
                        "aspectRatio": format!("{}:{}", w, h)
                    });
                }
            }
        }
    }

    if let Some(n) = req.n {
        gen_config["candidateCount"] = serde_json::json!(n);
    }

    // personGeneration: 根据 style/quality 推断
    if let Some(ref style) = req.style {
        if style == "natural" {
            gen_config["personGeneration"] = serde_json::json!("ALLOW_ADULT");
        }
    }

    // output_format → imageMimeType
    if let Some(ref fmt) = req.output_format {
        let mime = match fmt.as_str() {
            "png" => "image/png",
            "jpeg" | "jpg" => "image/jpeg",
            "webp" => "image/webp",
            _ => "image/png",
        };
        gen_config["imageConfig"] = {
            let mut ic = gen_config.get("imageConfig")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            ic["outputMimeType"] = serde_json::json!(mime);
            ic
        };
    }

    let mut payload = serde_json::json!({
        "contents": [{ "parts": [{ "text": req.prompt }] }],
        "generationConfig": gen_config
    });

    // 安全设置
    if let Some(ref moderation) = req.moderation {
        let threshold = match moderation.as_str() {
            "low" => "BLOCK_ONLY_HIGH",
            "auto" => "BLOCK_MEDIUM_AND_ABOVE",
            _ => "BLOCK_MEDIUM_AND_ABOVE",
        };
        payload["safetySettings"] = serde_json::json!([
            {
                "category": "HARM_CATEGORY_HARASSMENT",
                "threshold": threshold
            },
            {
                "category": "HARM_CATEGORY_HATE_SPEECH", 
                "threshold": threshold
            },
            {
                "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                "threshold": threshold
            },
            {
                "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                "threshold": threshold
            }
        ]);
    }

    payload
}

/// 将 Gemini generateContent 图片响应归一化为 OpenAI 标准格式
pub fn normalize_gemini_image_response(body: &serde_json::Value) -> serde_json::Value {
    let mut data = Vec::new();

    if let Some(candidates) = body.get("candidates").and_then(|c| c.as_array()) {
        for candidate in candidates {
            if let Some(parts) = candidate.pointer("/content/parts").and_then(|p| p.as_array()) {
                for part in parts {
                    if let Some(inline_data) = part.get("inlineData") {
                        let b64 = inline_data.get("data").and_then(|d| d.as_str()).unwrap_or("");
                        data.push(serde_json::json!({ "b64_json": b64 }));
                    }
                }
            }
        }
    }

    serde_json::json!({
        "created": chrono::Utc::now().timestamp(),
        "data": data
    })
}
