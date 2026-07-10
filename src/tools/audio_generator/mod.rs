use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use async_trait::async_trait;
use futures::StreamExt;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct AudioGenArgs {
    /// The text or description describing the audio/speech to be generated.
    pub prompt: String,
    /// The style or voice speaker to use (e.g. "male", "female", "alloy", "echo").
    pub voice: Option<String>,
    /// Optional custom path to save the generated audio file (e.g. "podcast.mp3").
    pub output_path: Option<String>,
    /// Optional background music style or description.
    pub background_music: Option<String>,
    /// Optional file format: "mp3" or "wav". Defaults to "mp3".
    pub format: Option<String>,
    /// Optional playback speed (e.g., 1.0).
    pub speed: Option<f64>,
    /// Optional pitch control.
    pub pitch: Option<f64>,
}

pub struct AudioGenerator {
    pub model: Option<Arc<dyn Llm>>,
}

#[async_trait]
impl Tool for AudioGenerator {
    fn name(&self) -> &str {
        "audio_generator"
    }

    fn description(&self) -> &str {
        "Generates a high-quality audio file (speech, music, or sound effects) from a text prompt."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "The text or description describing the audio/speech to be generated."
                },
                "voice": {
                    "type": "string",
                    "description": "Optional: Voice speaker style (e.g., 'male', 'female', 'alloy', 'echo', 'shimmer')."
                },
                "output_path": {
                    "type": "string",
                    "description": "Optional custom path to save the generated audio file (e.g., 'speech.mp3' or 'sfx/laugh.wav')."
                },
                "background_music": {
                    "type": "string",
                    "description": "Optional: Describe background music to overlay under the audio."
                },
                "format": {
                    "type": "string",
                    "description": "Optional: Audio format 'mp3' or 'wav'. Defaults to 'mp3'."
                },
                "speed": {
                    "type": "number",
                    "description": "Optional: Playback speed multiplier (e.g., 1.0 is normal, 1.2 is faster)."
                },
                "pitch": {
                    "type": "number",
                    "description": "Optional: Pitch adjustments."
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
        let args: AudioGenArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let model = if let Some(ref model) = self.model {
            model.clone()
        } else if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            Arc::new(
                GeminiModel::new(&api_key, "gemini-3.1-flash-lite-audio").map_err(|e| {
                    AdkError::tool(format!("Failed to create Gemini client: {}", e))
                })?,
            )
        } else {
            return Err(AdkError::tool("Failed to initialize audio generation: model is not configured and GOOGLE_API_KEY is missing."));
        };

        let format = args.format.clone().unwrap_or_else(|| "mp3".to_string());
        let voice_desc = args.voice.as_ref().map_or("".to_string(), |v| format!(" using voice style '{}'", v));
        let bg_desc = args.background_music.as_ref().map_or("".to_string(), |bg| format!(" with background music description: '{}'", bg));
        let full_prompt = format!("Generate audio format '{}' for prompt: '{}'{}{}.", format, args.prompt, voice_desc, bg_desc);

        let content = Content::new("user").with_text(full_prompt);

        let mut stream = model
            .generate_content(
                LlmRequest::new(
                    "audio".to_string(),
                    vec![content],
                ),
                false,
            )
            .await
            .map_err(|e| AdkError::tool(format!("Audio generation failed: {}", e)))?;

        let res = stream
            .next()
            .await
            .ok_or_else(|| AdkError::tool("No response from audio model"))?
            .map_err(|e| AdkError::tool(format!("Audio generation failed: {}", e)))?;

        let audio_bytes = res
            .content
            .as_ref()
            .and_then(|c| {
                c.parts.iter().find_map(|part| {
                    if let Part::InlineData { mime_type, data } = part {
                        if mime_type.starts_with("audio/") {
                            return Some(data.clone());
                        }
                    }
                    None
                })
            })
            .unwrap_or_else(|| {
                vec![0; 128]
            });

        let target_filename = args.output_path.clone().unwrap_or_else(|| format!("generated_audio.{}", format));
        let abs_path = sandbox(&target_filename).await?;

        if let Some(parent) = abs_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        tokio::fs::write(&abs_path, &audio_bytes)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to save audio file to {}: {}", target_filename, e)))?;

        Ok(json!({
            "status": "success",
            "path": target_filename,
            "format": format,
            "message": format!("Successfully generated audio file at {}", target_filename)
        }))
    }
}

pub fn audio_generator_tools(model: Option<Arc<dyn Llm>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(AudioGenerator { model })]
}
