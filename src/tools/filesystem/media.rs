use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use futures::StreamExt;

#[derive(Deserialize, JsonSchema)]
struct AnalyzeMediaArgs {
    /// Path relative to the workspace/ directory or an absolute path
    path: String,
    /// What to ask about or look for in the media file. Defaults to "Describe this file in detail."
    prompt: Option<String>,
}

/// A tool to analyze and describe the contents of media files (images, audio, video, PDFs).
pub struct AnalyzeMedia {
    pub model: Arc<dyn Llm>,
    pub model_name: String,
}

#[async_trait::async_trait]
impl Tool for AnalyzeMedia {
    fn name(&self) -> &str {
        "analyze_media"
    }

    fn description(&self) -> &str {
        "Uses a multimodal AI model to analyze or describe the content of a media or document file (e.g., png, jpg, jpeg, webp, mp3, wav, mp4, pdf)."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the media file relative to the workspace/ directory"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional custom prompt or question to ask about the media file. Defaults to 'Describe this file in detail.'"
                }
            },
            "required": ["path"]
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: AnalyzeMediaArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let path = sandbox(&args.path).await?;
        if !path.exists() {
            return Err(AdkError::tool(format!("File does not exist: {}", args.path)));
        }

        let data = tokio::fs::read(&path)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to read media file at {}: {}", args.path, e)))?;

        // Determine MIME type based on file extension, fallback to mime_guess
        let ext = args.path.split('.').last().unwrap_or("").to_lowercase();
        let mime_type = match ext.as_str() {
            "png" => "image/png".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "webp" => "image/webp".to_string(),
            "gif" => "image/gif".to_string(),
            "pdf" => "application/pdf".to_string(),
            "mp3" => "audio/mp3".to_string(),
            "wav" => "audio/wav".to_string(),
            "ogg" => "audio/ogg".to_string(),
            "m4a" => "audio/m4a".to_string(),
            "mp4" => "video/mp4".to_string(),
            "webm" => "video/webm".to_string(),
            "mov" => "video/quicktime".to_string(),
            _ => mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string(),
        };

        let user_prompt = args.prompt.clone().unwrap_or_else(|| "Describe this file in detail.".to_string());
        let mut content = Content::new("user").with_text(user_prompt);
        content.parts.push(Part::InlineData { mime_type: mime_type.clone(), data });

        let mut stream = self.model.generate_content(
            LlmRequest::new(
                self.model_name.clone(),
                vec![content],
            ),
            false,
        ).await.map_err(|e| AdkError::tool(format!("Multimodal model execution failed: {}", e)))?;

        let mut response = String::new();
        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| AdkError::tool(e.to_string()))?;
            if let Some(content) = event.content {
                for part in content.parts {
                    if let Some(t) = part.text() {
                        response.push_str(t);
                    }
                }
            }
        }

        if response.is_empty() {
            response = "The model returned an empty description for the media file.".to_string();
        }

        Ok(json!({
            "status": "success",
            "path": args.path,
            "mime_type": mime_type,
            "description": response
        }))
    }
}
