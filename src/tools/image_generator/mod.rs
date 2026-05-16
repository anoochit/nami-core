use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use base64::{engine::general_purpose, Engine as _};
use futures::StreamExt;
use schemars::JsonSchema;
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;

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

    async fn execute(&self, _context: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: ImagenArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let model = if let Some(ref model) = self.model {
            model.clone()
        } else {
            let api_key = std::env::var("GOOGLE_API_KEY")
                .map_err(|_| AdkError::tool("GOOGLE_API_KEY environment variable not set"))?;
            
            Arc::new(GeminiModel::new(&api_key, "gemini-2.5-flash-image")
                .map_err(|e| AdkError::tool(format!("Failed to create Gemini client: {}", e)))?)
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
        
        let res = stream.next().await
            .ok_or_else(|| AdkError::tool("No response from image model"))?
            .map_err(|e| AdkError::tool(format!("Image generation failed: {}", e)))?;

        // Extract image data from parts
        let image_bytes = res.content.as_ref()
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
