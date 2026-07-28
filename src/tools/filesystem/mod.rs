use crate::utils::{get_workspace_dir, sandbox};
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;


mod read;
mod write;
mod list;
mod media;

pub(crate) use read::*;
pub(crate) use write::*;
pub(crate) use list::*;
pub(crate) use media::*;

// ─── Tools ────────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub(crate) struct ReadFileArgs {
    /// Path relative to the workspace/ directory
    path: String,
    /// Optional encoding format: "text" or "base64". If omitted, auto-detects by trying text and falling back to base64.
    encoding: Option<String>,
    /// Optional 1-indexed start line (inclusive) for text files. Defaults to 1.
    start_line: Option<usize>,
    /// Optional 1-indexed end line (inclusive) for text files. Defaults to start_line + 799.
    end_line: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct WriteFileArgs {
    path: String,
    content: String,
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct PathArgs {
    /// Path relative to the workspace/ directory
    path: String,
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
pub(crate) struct ReplaceArgs {
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

#[derive(Deserialize, JsonSchema)]
pub(crate) struct GrepArgs {
    pattern: String,
    /// Optional subdirectory within workspace to restrict the search.
    path: Option<String>,
    /// Case-insensitive search. Defaults to false.
    case_insensitive: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct GlobArgs {
    pattern: String,
    /// Optional subdirectory within workspace to search
    cwd: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct MergeFilesArgs {
    /// A list of file paths relative to the workspace/ directory to merge.
    input_files: Vec<String>,
    /// The path to the destination file where the merged content will be saved.
    output_file: String,
    /// An optional string to insert between merged file contents (e.g. "\\n\\n---\\n\\n"). Defaults to a single newline.
    separator: Option<String>,
}

// ─── Registration ─────────────────────────────────────────────────────────────

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
