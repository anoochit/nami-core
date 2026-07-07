use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use futures::StreamExt;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct ImagenArgs {
    /// The text prompt describing the image to be generated.
    pub prompt: String,
    /// The aspect ratio for the image (e.g., "1:1", "16:9", "9:16"). Defaults to "1:1".
    pub aspect_ratio: Option<String>,
    /// Optional path to an existing image file (e.g., "cover.png") to use as a reference for generation.
    pub image_path: Option<String>,
    /// Optional custom path to save the generated image to (e.g., "mock.png" or "output/image.png").
    pub output_path: Option<String>,
}

pub struct ImageGenerator {
    pub model: Option<Arc<dyn Llm>>,
}

#[async_trait]
impl Tool for ImageGenerator {
    fn name(&self) -> &str {
        "image_generator"
    }

    fn description(&self) -> &str {
        "Generates a high-quality image from a text prompt."
    }

    async fn execute(
        &self,
        _context: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: ImagenArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let model = if let Some(ref model) = self.model {
            model.clone()
        } else if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            Arc::new(
                GeminiModel::new(&api_key, "gemini-3.1-flash-lite-image").map_err(|e| {
                    AdkError::tool(format!("Failed to create Gemini client: {}", e))
                })?,
            )
        } else {
            let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
                .or_else(|_| std::env::var("GCP_PROJECT"))
                .unwrap_or_default();
            if project_id.is_empty() {
                return Err(AdkError::tool(
                    "Failed to initialize image generation. GOOGLE_API_KEY is not set, \
                     and Vertex AI project ID (GOOGLE_CLOUD_PROJECT) is empty."
                ));
            }
            let location = std::env::var("GOOGLE_CLOUD_REGION")
                .or_else(|_| std::env::var("GCP_REGION"))
                .unwrap_or_else(|_| "global".to_string());

            Arc::new(
                GeminiModel::new_google_cloud_adc(&project_id, &location, "gemini-3.1-flash-lite-image").map_err(|e| {
                    AdkError::tool(format!(
                        "Failed to initialize image generation. GOOGLE_API_KEY is not set, \
                         and Vertex AI initialization failed: {}", e
                    ))
                })?,
            )
        };

        let mut prompt = args.prompt.clone();
        if let Some(ref ratio) = args.aspect_ratio {
            prompt = format!("{} Use aspect ratio {}.", prompt, ratio);
        }

        let mut content = Content::new("user").with_text(prompt.clone());

        let mut image_path = args.image_path.clone();
        if image_path.is_none() {
            if let Ok(re) = regex::Regex::new(r"([a-zA-Z0-9_\-./]+\.(?:png|jpg|jpeg|webp|gif))") {
                for cap in re.captures_iter(&prompt) {
                    let candidate = cap[1].to_string();
                    if let Ok(abs_path) = sandbox(&candidate).await {
                        if abs_path.exists() {
                            image_path = Some(candidate);
                            break;
                        }
                    }
                }
            }
        }

        if let Some(ref path) = image_path {
            let abs_path = sandbox(path).await?;
            let data = tokio::fs::read(&abs_path)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to read reference image at {}: {}", path, e)))?;
            let mime_type = if path.ends_with(".png") {
                "image/png".to_string()
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                "image/jpeg".to_string()
            } else if path.ends_with(".webp") {
                "image/webp".to_string()
            } else if path.ends_with(".gif") {
                "image/gif".to_string()
            } else {
                "image/png".to_string()
            };
            content.parts.push(Part::InlineData { mime_type, data });
        }

        let mut stream = model
            .generate_content(
                LlmRequest::new(
                    "image".to_string(),
                    vec![content],
                ),
                false,
            )
            .await
            .map_err(|e| AdkError::tool(format!("Image generation failed: {}", e)))?;

        let res = stream
            .next()
            .await
            .ok_or_else(|| AdkError::tool("No response from image model"))?
            .map_err(|e| AdkError::tool(format!("Image generation failed: {}", e)))?;

        // Extract image data from parts
        let image_bytes = res
            .content
            .as_ref()
            .and_then(|c| {
                c.parts.iter().find_map(|part| {
                    if let Part::InlineData { mime_type, data } = part {
                        if mime_type.starts_with("image/") {
                            if is_valid_image(data) {
                                return Some(Ok(data.clone()));
                            }
                            if let Ok(s) = std::str::from_utf8(data) {
                                return Some(extract_image_bytes(s));
                            }
                            return Some(Ok(data.clone()));
                        }
                    }
                    None
                })
            })
            .ok_or_else(|| AdkError::tool("No image data in response"))??;

        let mut output_path = args.output_path.clone();
        if output_path.is_none() {
            if let Ok(re) = regex::Regex::new(r"(?i)(?:save to|output to|as)\s+([a-zA-Z0-9_\-./]+\.(?:png|jpg|jpeg|webp|gif))") {
                if let Some(cap) = re.captures(&prompt) {
                    output_path = Some(cap[1].to_string());
                }
            }
        }

        let (_, display_name) = if let Some(ref custom_path) = output_path {
            let abs_path = sandbox(custom_path).await?;
            if let Some(parent) = abs_path.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            tokio::fs::write(&abs_path, &image_bytes)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to save image to custom path {}: {}", custom_path, e)))?;
            (abs_path, custom_path.clone())
        } else {
            let filename = format!("generated_{}.png", uuid::Uuid::new_v4());
            let output_dir = "generated";
            let abs_output_dir = sandbox(output_dir).await?;
            tokio::fs::create_dir_all(&abs_output_dir).await.ok();
            let dest_path = abs_output_dir.join(&filename);
            tokio::fs::write(&dest_path, &image_bytes)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to save image to disk: {}", e)))?;
            (dest_path, format!("{}/{}", output_dir, filename))
        };

        Ok(json!({
            "status": "success",
            "filename": display_name,
            "prompt": args.prompt
        }))
    }
}

pub fn image_generator_tools(model: Option<Arc<dyn Llm>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ImageGenerator { model })]
}

fn extract_image_bytes(data: &str) -> std::result::Result<Vec<u8>, AdkError> {
    // 1. Try robust base64 decode after filtering out any whitespaces/newlines
    let cleaned: String = data.chars().filter(|c| !c.is_whitespace()).collect();
    if let Ok(bytes) = general_purpose::STANDARD.decode(&cleaned) {
        if is_valid_image(&bytes) {
            return Ok(bytes);
        }
    }
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD_NO_PAD.decode(&cleaned) {
        if is_valid_image(&bytes) {
            return Ok(bytes);
        }
    }
    if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE.decode(&cleaned) {
        if is_valid_image(&bytes) {
            return Ok(bytes);
        }
    }
    if let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&cleaned) {
        if is_valid_image(&bytes) {
            return Ok(bytes);
        }
    }

    // 2. Fallback to raw bytes of the string (UTF-8 representation) if it contains the raw binary directly
    let raw_bytes = data.as_bytes().to_vec();
    if is_valid_image(&raw_bytes) {
        return Ok(raw_bytes);
    }

    // 3. Fallback to char-by-char cast (Latin1 / ISO-8859-1 format)
    let latin1_bytes: Vec<u8> = data.chars().map(|c| c as u8).collect();
    if is_valid_image(&latin1_bytes) {
        return Ok(latin1_bytes);
    }

    // If none of those match but base64 was successfully decoded, we still return the first decoded bytes
    if let Ok(bytes) = general_purpose::STANDARD.decode(&cleaned) {
        return Ok(bytes);
    }

    Err(AdkError::tool("Failed to decode image data as base64 or raw fallback bytes"))
}

fn is_valid_image(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    // PNG Magic: 89 50 4E 47
    if bytes[0] == 0x89 && bytes[1] == 0x50 && bytes[2] == 0x4E && bytes[3] == 0x47 {
        return true;
    }
    // JPEG Magic: FF D8 FF
    if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return true;
    }
    // GIF Magic: 47 49 46 ("GIF")
    if bytes[0] == 0x47 && bytes[1] == 0x49 && bytes[2] == 0x46 {
        return true;
    }
    // WebP Magic: RIFF .... WEBP
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return true;
    }
    false
}

#[cfg(test)]
mod tests;
