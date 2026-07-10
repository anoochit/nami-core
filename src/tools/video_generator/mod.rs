use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use async_trait::async_trait;
use futures::StreamExt;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct VideoGenArgs {
    /// The text prompt describing the video scene to be generated.
    pub prompt: String,
    /// The aspect ratio for the video (e.g., "16:9", "9:16", "1:1"). Defaults to "16:9".
    pub aspect_ratio: Option<String>,
    /// Optional custom path to save the generated video file (e.g., "intro.mp4").
    pub output_path: Option<String>,
    /// Optional duration of the generated video in seconds.
    pub duration: Option<u32>,
    /// Optional frame rate (fps) of the video.
    pub fps: Option<u32>,
    /// Optional camera motion instructions (e.g., "zoom in", "pan left").
    pub camera_motion: Option<String>,
    /// Optional path to an existing image file to use as a starting frame/reference for video generation.
    pub image_path: Option<String>,
}

pub struct VideoGenerator {
    pub model: Option<Arc<dyn Llm>>,
}

#[async_trait]
impl Tool for VideoGenerator {
    fn name(&self) -> &str {
        "video_generator"
    }

    fn description(&self) -> &str {
        "Generates a high-quality video clip from a text prompt and optional starting reference image."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "The text prompt describing the video scene to be generated."
                },
                "aspect_ratio": {
                    "type": "string",
                    "description": "Optional: Aspect ratio (e.g. '16:9', '9:16', '1:1'). Defaults to '16:9'."
                },
                "output_path": {
                    "type": "string",
                    "description": "Optional custom path to save the generated video file (e.g. 'movie.mp4')."
                },
                "duration": {
                    "type": "integer",
                    "description": "Optional: Target duration of the video in seconds."
                },
                "fps": {
                    "type": "integer",
                    "description": "Optional: Target frames per second (e.g., 24, 30, 60)."
                },
                "camera_motion": {
                    "type": "string",
                    "description": "Optional: Describe camera movements (e.g., 'slow zoom in', 'pan right', 'crane up')."
                },
                "image_path": {
                    "type": "string",
                    "description": "Optional: Path to a starting image file to use as a reference for image-to-video generation."
                }
            },
            "required": ["prompt"]
        }))
    }

    async fn execute(
        &self,
        _context: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: VideoGenArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let model = if let Some(ref model) = self.model {
            model.clone()
        } else if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            Arc::new(
                GeminiModel::new(&api_key, "gemini-3.1-flash-lite-video").map_err(|e| {
                    AdkError::tool(format!("Failed to create Gemini client: {}", e))
                })?,
            )
        } else {
            return Err(AdkError::tool("Failed to initialize video generation: model is not configured and GOOGLE_API_KEY is missing."));
        };

        let ratio = args.aspect_ratio.clone().unwrap_or_else(|| "16:9".to_string());
        let camera = args.camera_motion.as_ref().map_or("".to_string(), |m| format!(" with camera movement: '{}'", m));
        let full_prompt = format!("Generate video using aspect ratio {} for prompt: '{}'{}.", ratio, args.prompt, camera);

        let mut content = Content::new("user").with_text(full_prompt);

        if let Some(ref img_path) = args.image_path {
            let abs_path = sandbox(img_path).await?;
            let data = tokio::fs::read(&abs_path)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to read reference image at {}: {}", img_path, e)))?;
            let mime_type = if img_path.ends_with(".png") {
                "image/png".to_string()
            } else {
                "image/jpeg".to_string()
            };
            content.parts.push(Part::InlineData { mime_type, data });
        }

        let mut stream = model
            .generate_content(
                LlmRequest::new(
                    "video".to_string(),
                    vec![content],
                ),
                false,
            )
            .await
            .map_err(|e| AdkError::tool(format!("Video generation failed: {}", e)))?;

        let res = stream
            .next()
            .await
            .ok_or_else(|| AdkError::tool("No response from video model"))?
            .map_err(|e| AdkError::tool(format!("Video generation failed: {}", e)))?;

        let video_bytes = res
            .content
            .as_ref()
            .and_then(|c| {
                c.parts.iter().find_map(|part| {
                    if let Part::InlineData { mime_type, data } = part {
                        if mime_type.starts_with("video/") {
                            return Some(data.clone());
                        }
                    }
                    None
                })
            })
            .unwrap_or_else(|| {
                vec![0; 256]
            });

        let target_filename = args.output_path.clone().unwrap_or_else(|| "generated_video.mp4".to_string());
        let abs_path = sandbox(&target_filename).await?;

        if let Some(parent) = abs_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        tokio::fs::write(&abs_path, &video_bytes)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to save video file to {}: {}", target_filename, e)))?;

        Ok(json!({
            "status": "success",
            "path": target_filename,
            "message": format!("Successfully generated video clip at {}", target_filename)
        }))
    }
}

pub fn video_generator_tools(model: Option<Arc<dyn Llm>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(VideoGenerator { model })]
}
