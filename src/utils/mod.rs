use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;
use crate::utils::ignore::NamiIgnore;

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

/// Resolves a user-provided string into a safe path within the workspace.
pub async fn sandbox(user_path: &str) -> std::result::Result<PathBuf, AdkError> {
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

/// Helper to get the wiki directory path.
pub async fn get_wiki_dir() -> std::result::Result<PathBuf, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;
    let wiki_dir = root.join("wiki");
    if !wiki_dir.exists() {
        fs::create_dir_all(&wiki_dir)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create wiki directory: {}", e)))?;
    }
    Ok(wiki_dir)
}
