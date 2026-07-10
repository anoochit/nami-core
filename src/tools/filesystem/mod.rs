use crate::utils::{get_workspace_dir, sandbox};
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::fs;
use futures::StreamExt;

// ─── Tools ────────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct ReadFileArgs {
    /// Path relative to the workspace/ directory
    path: String,
    /// Optional encoding format: "text" or "base64". If omitted, auto-detects by trying text and falling back to base64.
    encoding: Option<String>,
    /// Optional 1-indexed start line (inclusive) for text files. Defaults to 1.
    start_line: Option<usize>,
    /// Optional 1-indexed end line (inclusive) for text files. Defaults to start_line + 799.
    end_line: Option<usize>,
}

/// Reads the contents of a file at the specified path within the workspace. Supports returning plain text or Base64-encoded multimedia files.
#[tool]
async fn read_file(args: ReadFileArgs) -> std::result::Result<Value, AdkError> {
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

#[derive(Deserialize, JsonSchema)]
struct WriteFileArgs {
    path: String,
    content: String,
}

/// Writes the provided content to a file at the specified path within the workspace. Creates parent directories if they do not exist.
#[tool]
async fn write_file(args: WriteFileArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;

    // Create parent dirs within workspace if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::write(&path, &args.content)
        .await
        .map_err(|e| AdkError::tool(format!("Write failed: {}", e)))?;

    Ok(json!({ "status": "success", "path": args.path }))
}

#[derive(Deserialize, JsonSchema)]
struct PathArgs {
    /// Path relative to the workspace/ directory
    path: String,
}

/// Lists the names of files and directories within the specified path in the workspace.
#[tool]
async fn list_dir(args: PathArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let mut dir = fs::read_dir(&path)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut entries = Vec::new();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        entries.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(json!({ "entries": entries }))
}

/// Deletes a file at the specified path within the workspace.
#[tool]
async fn delete_file(args: PathArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    if !path.exists() {
        return Err(AdkError::tool(format!("File does not exist: {}", args.path)));
    }
    if !path.is_file() {
        return Err(AdkError::tool(format!("Path is not a file: {}", args.path)));
    }

    fs::remove_file(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Delete failed: {}", e)))?;

    Ok(json!({ "status": "success", "path": args.path }))
}


#[derive(Deserialize, JsonSchema)]
pub(crate) struct ExecArgs {
    pub(crate) command: String,
    /// Optional subdirectory within workspace
    pub(crate) cwd: Option<String>,
    /// Optional stdin input for the command
    pub(crate) input: Option<String>,
}

/// Executes a shell command within the workspace.
#[tool]
pub(crate) async fn exec_command(args: ExecArgs) -> std::result::Result<Value, AdkError> {
    use std::process::Stdio;
    use tokio::process::Command;

    let root: std::path::PathBuf = get_workspace_dir().await?;
    let run_dir = match args.cwd {
        Some(c) => sandbox(&c).await?,
        None => root.clone(),
    };

    #[cfg(target_os = "windows")]
    let mut command = Command::new("cmd.exe");
    #[cfg(target_os = "windows")]
    command.arg("/C");

    #[cfg(not(target_os = "windows"))]
    let mut command = Command::new("sh");
    #[cfg(not(target_os = "windows"))]
    command.arg("-c");

    let mut child = command
        .arg(&args.command)
        .current_dir(&run_dir)
        // Set HOME to workspace to prevent tools from leaking into the host system
        .env("HOME", &root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AdkError::tool(e.to_string()))?;

    if let Some(input) = args.input {
        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| AdkError::tool(format!("Failed to write to stdin: {}", e)))?;
            stdin
                .flush()
                .await
                .map_err(|e| AdkError::tool(format!("Failed to flush stdin: {}", e)))?;
        }
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;

    Ok(json!({
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "exit_code": output.status.code()
    }))
}

#[derive(Deserialize, JsonSchema)]
struct ReplaceArgs {
    path: String,
    old_string: String,
    new_string: String,
    /// If true, multiple occurrences of `old_string` will be replaced. If false (default), multiple occurrences will return an error to prevent accidental collateral edits.
    allow_multiple: Option<bool>,
    /// Optional 1-indexed start line (inclusive) to limit the search/replace range.
    start_line: Option<usize>,
    /// Optional 1-indexed end line (inclusive) to limit the search/replace range.
    end_line: Option<usize>,
}

/// Replaces occurrences of `old_string` with `new_string` in a specified file with optional boundaries and occurrence checks.
#[tool]
async fn replace_text(args: ReplaceArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;

    let lines: Vec<&str> = content.split('\n').collect();
    let total_lines = lines.len();

    let start_line = args.start_line.unwrap_or(1);
    if start_line == 0 {
        return Err(AdkError::tool("start_line must be 1-indexed (greater than 0)".to_string()));
    }

    let start_idx = (start_line - 1).min(total_lines);
    let end_idx = match args.end_line {
        Some(e) => e.min(total_lines),
        None => total_lines,
    };

    if start_idx > end_idx {
        return Err(AdkError::tool("start_line cannot be greater than end_line".to_string()));
    }

    let pre_lines = &lines[0..start_idx];
    let target_lines = &lines[start_idx..end_idx];
    let post_lines = &lines[end_idx..];

    let target_block = target_lines.join("\n");

    let matches: Vec<_> = target_block.matches(&args.old_string).collect();
    let count = matches.len();

    if count == 0 {
        return Err(AdkError::tool("old_string not found in the specified line range".to_string()));
    }

    let allow_multiple = args.allow_multiple.unwrap_or(false);
    if count > 1 && !allow_multiple {
        return Err(AdkError::tool(format!(
            "Found {} occurrences of old_string in the specified line range, but allow_multiple is false",
            count
        )));
    }

    let new_target_block = if allow_multiple {
        target_block.replace(&args.old_string, &args.new_string)
    } else {
        target_block.replacen(&args.old_string, &args.new_string, 1)
    };

    let mut final_lines = Vec::new();
    final_lines.extend(pre_lines.iter().copied());
    final_lines.push(&new_target_block);
    final_lines.extend(post_lines.iter().copied());

    let final_content = final_lines.join("\n");

    fs::write(&path, final_content)
        .await
        .map_err(|e| AdkError::tool(format!("Write failed: {}", e)))?;

    Ok(json!({ "status": "success", "replaced_occurrences": count }))
}

#[derive(Deserialize, JsonSchema)]
struct GrepArgs {
    pattern: String,
    /// Optional subdirectory within workspace to restrict the search.
    path: Option<String>,
    /// Case-insensitive search. Defaults to false.
    case_insensitive: Option<bool>,
}

/// Searches recursively for a regular expression pattern within text files natively.
#[tool]
async fn grep_search(args: GrepArgs) -> std::result::Result<Value, AdkError> {
    let search_root = match args.path {
        Some(p) => sandbox(&p).await?,
        None => get_workspace_dir().await?,
    };

    let is_case_insensitive = args.case_insensitive.unwrap_or(false);
    let re = regex::RegexBuilder::new(&args.pattern)
        .case_insensitive(is_case_insensitive)
        .build()
        .map_err(|e| AdkError::tool(format!("Invalid regex pattern: {}", e)))?;

    let mut results = Vec::new();
    let mut total_matches = 0;
    let max_matches = 100; // safety limit to keep tokens safe

    for entry in walkdir::WalkDir::new(&search_root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "target" && name != "node_modules" && name != ".gemini" && name != "build" && name != "dist"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            // Let's read the file contents
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // If it contains null bytes, it's probably binary. Skip it.
                if content.contains('\0') {
                    continue;
                }
                
                let rel_path = file_path.strip_prefix(&search_root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                for (idx, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        total_matches += 1;
                        if results.len() < max_matches {
                            results.push(json!({
                                "path": rel_path.clone(),
                                "line": idx + 1,
                                "content": line.to_string(),
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "results": results,
        "total_matches": total_matches,
        "truncated": total_matches > max_matches,
    }))
}

#[derive(Deserialize, JsonSchema)]
struct GlobArgs {
    pattern: String,
    /// Optional subdirectory within workspace to search
    cwd: Option<String>,
}

/// Finds files matching a specific glob pattern within the workspace natively.
#[tool]
async fn glob_find(args: GlobArgs) -> std::result::Result<Value, AdkError> {
    let search_root = match args.cwd {
        Some(c) => sandbox(&c).await?,
        None => get_workspace_dir().await?,
    };

    let glob = globset::Glob::new(&args.pattern)
        .map_err(|e| AdkError::tool(format!("Invalid glob pattern: {}", e)))?
        .compile_matcher();

    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(&search_root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "target" && name != "node_modules" && name != ".gemini"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(&search_root) {
                if glob.is_match(rel_path) {
                    files.push(rel_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(json!({ "files": files }))
}

#[derive(Deserialize, JsonSchema)]
struct MergeFilesArgs {
    /// A list of file paths relative to the workspace/ directory to merge.
    input_files: Vec<String>,
    /// The path to the destination file where the merged content will be saved.
    output_file: String,
    /// An optional string to insert between merged file contents (e.g. "\\n\\n---\\n\\n"). Defaults to a single newline.
    separator: Option<String>,
}

/// Reads multiple files and concatenates their contents into a single output file.
#[tool]
async fn merge_files(args: MergeFilesArgs) -> std::result::Result<Value, AdkError> {
    let mut combined_content = String::new();
    let separator = args.separator.unwrap_or_else(|| "\n".to_string());

    for (index, file_path) in args.input_files.iter().enumerate() {
        let path = sandbox(file_path).await?;

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to read {}: {}", file_path, e)))?;

        combined_content.push_str(&content);

        if index < args.input_files.len() - 1 {
            combined_content.push_str(&separator);
        }
    }

    let out_path = sandbox(&args.output_file).await?;

    // Create parent dirs within workspace if they don't exist
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::write(&out_path, combined_content).await.map_err(|e| {
        AdkError::tool(format!(
            "Failed to write merged output to {}: {}",
            args.output_file, e
        ))
    })?;

    Ok(
        json!({ "status": "success", "message": format!("Merged {} files into {}", args.input_files.len(), args.output_file) }),
    )
}

// ─── Registration ─────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct AnalyzeMediaArgs {
    /// Path relative to the workspace/ directory or an absolute path
    path: String,
    /// What to ask about or look for in the media file. Defaults to "Describe this file in detail."
    prompt: Option<String>,
}

/// A tool to analyze and describe the contents of media files (images, audio, video, PDFs).
pub struct AnalyzeMedia {
    model: Arc<dyn Llm>,
    model_name: String,
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

pub fn filesystem_tools(model: Arc<dyn Llm>, model_name: String) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ReadFile),
        Arc::new(WriteFile),
        Arc::new(DeleteFile),
        Arc::new(ListDir),
        Arc::new(ExecCommand),
        Arc::new(ReplaceText),
        Arc::new(GrepSearch),  
        Arc::new(GlobFind),
        Arc::new(MergeFiles),
        Arc::new(AnalyzeMedia { model, model_name }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_rw_operations() {
        // Write to a temporary file in the workspace
        let write_args = WriteFileArgs {
            path: "test_file.txt".to_string(),
            content: "hello world".to_string(),
        };
        write_file(write_args).await.unwrap();

        // Read the file back
        let read_args = ReadFileArgs {
            path: "test_file.txt".to_string(),
            encoding: None,
            start_line: None,
            end_line: None,
        };
        let read_result = read_file(read_args).await.unwrap();
        assert_eq!(read_result["content"], "hello world");
        assert_eq!(read_result["encoding"], "text");
        assert_eq!(read_result["mime_type"], "text/plain");

        // Test explicit base64 encoding read
        let read_args_b64 = ReadFileArgs {
            path: "test_file.txt".to_string(),
            encoding: Some("base64".to_string()),
            start_line: None,
            end_line: None,
        };
        let read_result_b64 = read_file(read_args_b64).await.unwrap();
        use base64::Engine as _;
        let decoded_bytes = base64::prelude::BASE64_STANDARD.decode(read_result_b64["content"].as_str().unwrap()).unwrap();
        assert_eq!(String::from_utf8(decoded_bytes).unwrap(), "hello world");
        assert_eq!(read_result_b64["encoding"], "base64");

        // Replace text
        let replace_args = ReplaceArgs {
            path: "test_file.txt".to_string(),
            old_string: "world".to_string(),
            new_string: "gemini".to_string(),
            allow_multiple: None,
            start_line: None,
            end_line: None,
        };
        replace_text(replace_args).await.unwrap();

        // Verify replacement
        let read_args_after = ReadFileArgs {
            path: "test_file.txt".to_string(),
            encoding: None,
            start_line: None,
            end_line: None,
        };
        let read_result_after = read_file(read_args_after).await.unwrap();
        assert_eq!(read_result_after["content"], "hello gemini");
    }
}
