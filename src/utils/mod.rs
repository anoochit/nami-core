use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;

pub mod ignore;

const WORKSPACE_NAME: &str = "workspace";

/// Returns the absolute path to the sandbox directory.
/// Ensures the directory exists on disk.
pub async fn get_workspace_dir() -> std::result::Result<PathBuf, AdkError> {
    let current_dir = std::env::current_dir()
        .map_err(|e| AdkError::tool(format!("Failed to get current directory: {}", e)))?;

    let root = current_dir.join(WORKSPACE_NAME);

    if !root.exists() {
        fs::create_dir_all(&root)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create workspace: {}", e)))?;
    }

    // Canonicalize for security checks
    Ok(fs::canonicalize(&root).await.unwrap_or(root))
}
