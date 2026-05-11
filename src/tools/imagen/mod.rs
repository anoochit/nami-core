use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use crate::utils::sandbox;
use crate::tools::filesystem::exec_command; // Reuse exec_command

#[derive(Deserialize, JsonSchema)]
struct ImagenArgs {
    /// The prompt for the image generation.
    prompt: String,
    /// Aspect ratio, defaults to "1:1".
    aspect_ratio: Option<String>,
}

/// Generates an image using the Imagen model and saves it to workspace/generated/.
#[tool]
async fn imagen(args: ImagenArgs) -> std::result::Result<Value, AdkError> {
    let script_path = "workspace/.skills/imagen/scripts/generate_image.py";
    let output_dir = "workspace/generated";
    
    // Ensure output directory exists
    let abs_output_dir = sandbox(output_dir).await?;
    tokio::fs::create_dir_all(&abs_output_dir).await.ok();

    // Prepare arguments for the script
    let prompt = args.prompt;
    let aspect_ratio = args.aspect_ratio.unwrap_or_else(|| "1:1".to_string());
    
    let input_json = json!({
        "prompt": prompt,
        "aspect_ratio": aspect_ratio
    }).to_string();

    // Execute the Python script
    let cmd = format!("python3 {}", script_path);
    
    // We reuse filesystem::exec_command with the new input field
    let result = exec_command(crate::tools::filesystem::ExecArgs {
        command: cmd,
        cwd: None,
        input: Some(input_json),
    }).await?;
    
    // Process result
    let stdout = result["stdout"].as_str().unwrap_or("");
    let parsed: Value = serde_json::from_str(stdout).map_err(|e| AdkError::tool(format!("Failed to parse script output: {}", e)))?;
    
    if parsed["status"] == "success" {
        let image_path = parsed["image_path"].as_str().unwrap_or("");
        // Move image to workspace/generated/
        let source_path = sandbox(image_path).await?;
        let dest_path = abs_output_dir.join(image_path);
        
        tokio::fs::rename(source_path, dest_path).await.map_err(|e| AdkError::tool(format!("Failed to move image: {}", e)))?;
        
        Ok(json!({ "filename": format!("workspace/generated/{}", image_path) }))
    } else {
        Err(AdkError::tool(parsed["message"].as_str().unwrap_or("Unknown error").to_string()))
    }
}

pub fn imagen_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(Imagen)]
}
