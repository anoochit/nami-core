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

        let mut stream = model
            .generate_content(
                LlmRequest::new(
                    "image".to_string(),
                    vec![Content::new("user").with_text(prompt)],
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
                            return Some(general_purpose::STANDARD.decode(data));
                        }
                    }
                    None
                })
            })
            .ok_or_else(|| AdkError::tool("No image data in response"))?
            .map_err(|e| AdkError::tool(format!("Failed to decode image base64: {}", e)))?;

        let filename = format!("generated_{}.png", uuid::Uuid::new_v4());
        let output_dir = "generated";
        let abs_output_dir = sandbox(output_dir).await?;
        tokio::fs::create_dir_all(&abs_output_dir).await.ok();

        tokio::fs::write(abs_output_dir.join(&filename), &image_bytes)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to save image to disk: {}", e)))?;

        Ok(json!({
            "status": "success",
            "filename": format!("{}/{}", output_dir, filename),
            "prompt": args.prompt
        }))
    }
}

pub fn image_generator_tools(model: Option<Arc<dyn Llm>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ImageGenerator { model })]
}

#[cfg(test)]
mod tests;
