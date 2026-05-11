use crate::utils::sandbox;
use adk_gemini::{Gemini, Part};
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use base64::{Engine as _, engine::general_purpose};
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

/// Generates a high-quality image from a text prompt.
///
/// This tool takes a prompt and optional aspect ratio, generates an image, and saves it
/// as a PNG file in the `generated/` directory within the sandbox.
#[tool]
async fn image_generator(args: ImagenArgs) -> std::result::Result<Value, AdkError> {
    let api_key: String = std::env::var("GOOGLE_API_KEY")
        .map_err(|_| AdkError::tool("GOOGLE_API_KEY environment variable not set"))?;

    // From the README: image generation uses a Flash/image-preview model
    let model = "models/gemini-2.5-flash-image-preview".to_string();
    let client = Gemini::with_model(api_key, model)
        .map_err(|e| AdkError::tool(format!("Failed to create Gemini client: {}", e)))?;

    let mut prompt = args.prompt.clone();
    if let Some(ref ratio) = args.aspect_ratio {
        prompt = format!("{} Use aspect ratio {}.", prompt, ratio);
    }

    let res = client
        .generate_content()
        .with_user_message(prompt)
        .execute()
        .await
        .map_err(|e| AdkError::tool(format!("Image generation failed: {}", e)))?;

    // From the README: parts is Option<Vec<Part>>, so unwrap it first.
    // The image variant is Part::InlineData { inline_data } where
    // inline_data.data is base64-encoded and inline_data.mime_type identifies it.
    let image_bytes = res
        .candidates
        .iter()
        .find_map(|c| c.content.parts.as_ref())
        .and_then(|parts| {
            parts.iter().find_map(|part| {
                if let Part::InlineData { inline_data } = part {
                    if inline_data.mime_type.starts_with("image/") {
                        return Some(general_purpose::STANDARD.decode(&inline_data.data));
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

pub fn image_generator_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ImageGenerator)]
}
