use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;
use crate::utils::ignore::NamiIgnore;
use serde_json;

pub mod ignore;

const WORKSPACE_NAME: &str = "workspace";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCategory {
    Transient, // Retryable (e.g., rate limits, timeouts)
    Fatal,     // Non-retryable (e.g., authentication, bad schema)
}

/// Categorizes an error based on its content to determine if it's transient and retryable.
pub fn categorize_error(e: &anyhow::Error) -> ErrorCategory {
    let err_str = e.to_string().to_lowercase();
    if err_str.contains("rate_limited") || 
       err_str.contains("429") || 
       err_str.contains("timeout") ||
       err_str.contains("408") ||
       err_str.contains("503") ||
       err_str.contains("529") ||
       (err_str.contains("400") && err_str.contains("number of function response parts")) {
        ErrorCategory::Transient
    } else {
        ErrorCategory::Fatal
    }
}

/// Returns a clean, user-friendly error message by stripping technical details and parsing JSON.
pub fn clean_error_message(e: impl std::fmt::Display) -> String {
    let err_str = e.to_string();

    if err_str.contains("insufficient_quota") {
        return "API Quota Exceeded: You have exceeded your OpenAI quota. Please check your plan and billing details.".to_string();
    }

    if err_str.contains("rate_limited") || err_str.contains("429 Too Many Requests") {
        return "Rate Limit Reached: The AI provider is currently rate limiting requests. Please wait a moment before trying again.".to_string();
    }

    if err_str.contains("invalid_api_key") || err_str.contains("401 Unauthorized") {
        return "Invalid API Key: The API key provided is invalid or has expired. Please check your configuration.".to_string();
    }

    // Try to extract a clean message from common error patterns
    let mut clean_msg = err_str.clone();

    // If it contains a JSON error from a provider, try to parse it
    if let Some(json_start) = err_str.find('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&err_str[json_start..]) {
            if let Some(msg) = v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                clean_msg = msg.to_string();
            } else if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                clean_msg = msg.to_string();
            }
        }
    }

    // If it's the specific format ADK uses, try to strip the prefix
    if clean_msg.contains("error=") {
        if let Some(idx) = clean_msg.rfind("error=") {
            clean_msg = clean_msg[idx + 6..].to_string();
        }
    }

    // Strip any remaining JSON-like trailing parts if we didn't parse them
    if let Some(idx) = clean_msg.find("): {") {
        clean_msg = clean_msg[..idx].to_string();
    }

    clean_msg.trim().to_string()
}

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
