use crate::utils::{get_workspace_dir, ignore::NamiIgnore};
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

/// Resolves a user-provided string into a safe path within the workspace.
async fn sandbox(user_path: &str) -> std::result::Result<PathBuf, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;

    // 1. Clean the user path: remove leading slashes and drive letters (Windows)
    // to prevent the join from treating it as a new absolute path.
    let clean_path = user_path.trim_start_matches(['/', '\\']);

    // 2. Join and normalize
    let mut joined = root.clone();
    joined.push(clean_path);

    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            c => normalized.push(c),
        }
    }

    // 3. Final Guard: The resulting path MUST still start with the workspace root.
    if !normalized.starts_with(&root) {
        return Err(AdkError::tool(format!(
            "Security Error: Path '{}' attempts to escape sandbox.",
            user_path
        )));
    }

    // 4. .namiignore Check
    let relative_to_root = normalized.strip_prefix(&root).unwrap_or(&normalized);
    let ignore = NamiIgnore::load().await;
    if ignore.is_ignored(relative_to_root) {
        return Err(AdkError::tool(format!(
            "Access Denied: Path '{}' is ignored by .namiignore policy.",
            user_path
        )));
    }

    Ok(normalized)
}

// ─── Tools ────────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct PathArgs {
    /// Path relative to the workspace/ directory
    path: String,
}

/// Reads the contents of a file at the specified path within the workspace.
#[tool]
async fn read_file(args: PathArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;

    Ok(json!({ "content": content }))
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

#[derive(Deserialize, JsonSchema)]
struct ExecArgs {
    command: String,
    /// Optional subdirectory within workspace
    cwd: Option<String>,
}

/// Executes a shell command within the workspace.
#[tool]
async fn exec_command(args: ExecArgs) -> std::result::Result<Value, AdkError> {
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

    let output = command
        .arg(&args.command)
        .current_dir(&run_dir)
        // Set HOME to workspace to prevent tools from leaking into the host system
        .env("HOME", &root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AdkError::tool(e.to_string()))?
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
}

/// Replaces all occurrences of `old_string` with `new_string` in a specified file.
#[tool]
async fn replace_text(args: ReplaceArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;

    if !content.contains(&args.old_string) {
        return Err(AdkError::tool("Old string not found in file".to_string()));
    }

    let new_content = content.replace(&args.old_string, &args.new_string);
    fs::write(&path, new_content)
        .await
        .map_err(|e| AdkError::tool(format!("Write failed: {}", e)))?;

    Ok(json!({ "status": "success" }))
}

#[derive(Deserialize, JsonSchema)]
struct GrepArgs {
    pattern: String,
    _include_pattern: Option<String>,
}

/// Searches for a regular expression pattern within files in the workspace.
#[tool]
async fn grep_search(args: GrepArgs) -> std::result::Result<Value, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;
    let mut command = Command::new("grep");
    // Ensure the grep command operates strictly within the workspace root
    command.arg("-r").arg(&args.pattern).arg(".");

    let output = command
        .current_dir(&root)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| AdkError::tool(e.to_string()))?
        .wait_with_output()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;

    Ok(json!({ "results": String::from_utf8_lossy(&output.stdout) }))
}

#[derive(Deserialize, JsonSchema)]
struct GlobArgs {
    pattern: String,
    /// Optional subdirectory within workspace to search
    cwd: Option<String>,
}

/// Finds files matching a specific glob pattern within the workspace.
#[tool]
async fn glob_find(args: GlobArgs) -> std::result::Result<Value, AdkError> {
    let search_root = match args.cwd {
        Some(c) => sandbox(&c).await?,
        None => get_workspace_dir().await?,
    };

    let mut command = Command::new("find");
    // Use -wholename to allow path separators in the pattern
    command
        .arg(&search_root)
        .arg("-wholename")
        .arg(format!("*{}*", &args.pattern));

    let output = command
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| AdkError::tool(e.to_string()))?
        .wait_with_output()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;

    Ok(json!({ "files": String::from_utf8_lossy(&output.stdout) }))
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

pub fn filesystem_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ReadFile),
        Arc::new(WriteFile),
        Arc::new(ListDir),
        Arc::new(ExecCommand),
        Arc::new(ReplaceText),
        Arc::new(GrepSearch),
        Arc::new(GlobFind),
        Arc::new(MergeFiles),
    ]
}
