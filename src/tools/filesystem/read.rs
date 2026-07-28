use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_tool::tool;
use serde_json::{Value, json};
use tokio::fs;
use super::ReadFileArgs;

#[tool]
pub(crate) async fn read_file(args: ReadFileArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    
    // Get MIME type based on file extension
    let mime_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    let encoding_req = args.encoding.as_deref().unwrap_or("auto");

    match encoding_req {
        "text" => {
            let content = fs::read_to_string(&path)
                .await
                .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;
            
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();

            let start_line = args.start_line.unwrap_or(1);
            let end_line = args.end_line.unwrap_or(start_line + 799);

            if start_line == 0 {
                return Err(AdkError::tool("start_line must be 1-indexed (greater than 0)".to_string()));
            }
            if end_line < start_line {
                return Err(AdkError::tool("end_line must be greater than or equal to start_line".to_string()));
            }

            if total_lines == 0 {
                return Ok(json!({
                    "content": "",
                    "encoding": "text",
                    "mime_type": mime_type,
                    "start_line": start_line,
                    "end_line": end_line,
                    "total_lines": 0,
                    "truncated": false
                }));
            }

            let start_idx = (start_line - 1).min(total_lines - 1);
            let end_idx = (end_line - 1).min(total_lines - 1);

            let sliced_lines = &lines[start_idx..=end_idx];
            let joined = sliced_lines.join("\n");
            let truncated = total_lines > (end_idx + 1) || start_idx > 0;

            Ok(json!({
                "content": joined,
                "encoding": "text",
                "mime_type": mime_type,
                "start_line": start_idx + 1,
                "end_line": end_idx + 1,
                "total_lines": total_lines,
                "truncated": truncated
            }))
        }
        "base64" => {
            let bytes = fs::read(&path)
                .await
                .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;
            use base64::Engine as _;
            let b64 = base64::prelude::BASE64_STANDARD.encode(&bytes);
            Ok(json!({
                "content": b64,
                "encoding": "base64",
                "mime_type": mime_type
            }))
        }
        _ => {
            // Auto-detect: try reading as UTF-8 first
            match fs::read_to_string(&path).await {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    let total_lines = lines.len();

                    let start_line = args.start_line.unwrap_or(1);
                    let end_line = args.end_line.unwrap_or(start_line + 799);

                    if start_line == 0 {
                        return Err(AdkError::tool("start_line must be 1-indexed (greater than 0)".to_string()));
                    }
                    if end_line < start_line {
                        return Err(AdkError::tool("end_line must be greater than or equal to start_line".to_string()));
                    }

                    if total_lines == 0 {
                        return Ok(json!({
                            "content": "",
                            "encoding": "text",
                            "mime_type": mime_type,
                            "start_line": start_line,
                            "end_line": end_line,
                            "total_lines": 0,
                            "truncated": false
                        }));
                    }

                    let start_idx = (start_line - 1).min(total_lines - 1);
                    let end_idx = (end_line - 1).min(total_lines - 1);

                    let sliced_lines = &lines[start_idx..=end_idx];
                    let joined = sliced_lines.join("\n");
                    let truncated = total_lines > (end_idx + 1) || start_idx > 0;

                    Ok(json!({
                        "content": joined,
                        "encoding": "text",
                        "mime_type": mime_type,
                        "start_line": start_idx + 1,
                        "end_line": end_idx + 1,
                        "total_lines": total_lines,
                        "truncated": truncated
                    }))
                }
                Err(_) => {
                    // Fallback to reading as binary and encoding to base64
                    let bytes = fs::read(&path)
                        .await
                        .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;
                    use base64::Engine as _;
                    let b64 = base64::prelude::BASE64_STANDARD.encode(&bytes);
                    Ok(json!({
                        "content": b64,
                        "encoding": "base64",
                        "mime_type": mime_type
                    }))
                }
            }
        }
    }
}
