use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;
use crate::utils::ignore::NamiIgnore;
use serde_json;
use tokio::time::{sleep, Duration};

pub mod ignore;
pub mod gemini_cache;
pub mod markdown;

pub static HTTP_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

pub fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}



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

/// Executes a future with truncated exponential backoff and jitter for transient errors.
pub async fn with_retry<F, Fut, T>(
    name: &str,
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
    max_delay: Duration,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut delay = initial_delay;
    
    for i in 0..=max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) if i < max_retries && categorize_error(&e) == ErrorCategory::Transient => {
                let jitter = Duration::from_millis(rand::random::<u64>() % 200);
                let current_delay = delay + jitter;
                
                log::warn!(
                    "[{}] Transient error (retry {}/{}): {}. Retrying in {:?}...",
                    name,
                    i + 1,
                    max_retries,
                    e,
                    current_delay
                );
                
                sleep(current_delay).await;
                
                // Exponential backoff truncated at max_delay
                delay = std::cmp::min(delay * 2, max_delay);
            }
            Err(e) => return Err(e),
        }
    }
    
    // This part should technically not be reached because the loop returns Err(e) on the last attempt
    Err(anyhow::anyhow!("Retry limit exceeded for {}", name))
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

/// Returns the path to the global Nami configuration and state directory (`~/.nami`).
/// Creates the directory on disk if it does not exist.
pub fn get_nami_dir() -> PathBuf {
    let path = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nami");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
}

pub fn clean_unc_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        if path_str.starts_with(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{}", &path_str[8..]))
        } else if path_str.starts_with(r"\\?\") {
            PathBuf::from(&path_str[4..])
        } else {
            path
        }
    }
    #[cfg(not(windows))]
    {
        path
    }
}

/// Returns the absolute path to the workspace directory.
/// Ensures the directory exists on disk.
pub async fn get_workspace_dir() -> std::result::Result<PathBuf, AdkError> {
    // 1. Check environment variable override
    if let Ok(env_ws) = std::env::var("NAMI_WORKSPACE") {
        if !env_ws.is_empty() {
            let path = PathBuf::from(env_ws);
            let absolute = clean_unc_path(fs::canonicalize(&path).await.unwrap_or(path));
            return Ok(absolute);
        }
    }

    // 2. Always default to current working directory
    let current_dir = std::env::current_dir()
        .map_err(|e| AdkError::tool(format!("Failed to get current directory: {}", e)))?;
    let canonical_current = clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir.clone()));

    Ok(canonical_current)
}

/// Resolves a user-provided string into a safe path within the workspace.
pub async fn sandbox(user_path: &str) -> std::result::Result<PathBuf, AdkError> {
    sandbox_with_ignore(user_path, None).await
}

pub async fn sandbox_with_ignore(user_path: &str, ignore: Option<&NamiIgnore>) -> std::result::Result<PathBuf, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;

    let mut user_path_buf = clean_unc_path(PathBuf::from(user_path));
    if user_path_buf.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            if let Ok(stripped) = user_path_buf.strip_prefix("~") {
                user_path_buf = home.join(stripped);
            }
        }
    }

    let mut normalized;
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let nami_dir = home.join(".nami");
    let agents_dir = home.join(".agents");

    if user_path_buf.is_absolute() {
        // If it's absolute, check if it falls within the workspace root or allowed directories
        if user_path_buf.starts_with(&root) || user_path_buf.starts_with(&nami_dir) || user_path_buf.starts_with(&agents_dir) {
            normalized = user_path_buf;
        } else {
            return Err(AdkError::tool(format!(
                "Security Error: Absolute path '{}' attempts to escape sandbox '{}'.",
                user_path, root.display()
            )));
        }
    } else {
        // If it is relative, we check if it starts with the last component of root (e.g. "x/")
        // to prevent duplicate nesting (like resolving /path/to/x/x/file.txt instead of /path/to/x/file.txt)
        let mut clean_p = user_path_buf;
        if let Some(root_name) = root.file_name() {
            if clean_p.starts_with(root_name) {
                if let Ok(stripped) = clean_p.strip_prefix(root_name) {
                    clean_p = stripped.to_path_buf();
                }
            }
        }
        normalized = root.join(clean_p);
    }

    // Normalize components to resolve any dot/parent-dir traversal (e.g., /a/b/../c)
    let mut clean_normalized = PathBuf::new();
    for component in normalized.components() {
        match component {
            std::path::Component::ParentDir => {
                clean_normalized.pop();
            }
            std::path::Component::CurDir => {}
            c => clean_normalized.push(c),
        }
    }
    normalized = clean_unc_path(clean_normalized);

    // 3. Final Guard: The resulting path MUST still start with the workspace root or allowed directories.
    if !normalized.starts_with(&root) && !normalized.starts_with(&nami_dir) && !normalized.starts_with(&agents_dir) {
        return Err(AdkError::tool(format!(
            "Security Error: Path '{}' attempts to escape sandbox.",
            user_path
        )));
    }

    // 4. .namiignore Check
    let relative_to_root = normalized.strip_prefix(&root).unwrap_or(&normalized);
    
    let is_ignored = if let Some(i) = ignore {
        i.is_ignored(relative_to_root)
    } else {
        let i = NamiIgnore::load().await;
        i.is_ignored(relative_to_root)
    };

    if is_ignored {
        return Err(AdkError::tool(format!(
            "Access Denied: Path '{}' is ignored by .namiignore policy.",
            user_path
        )));
    }

    Ok(normalized)
}

/// Helper to get the wiki directory path.
pub async fn get_wiki_dir() -> std::result::Result<PathBuf, AdkError> {
    let wiki_dir = get_nami_dir().join("wiki");
    if !wiki_dir.exists() {
        fs::create_dir_all(&wiki_dir)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create wiki directory: {}", e)))?;
    }
    Ok(wiki_dir)
}

pub fn save_agent_statistic(provider: &str, model_name: &str, duration_secs: f64, total_tokens: usize) {
    let stats_path = get_nami_dir().join("stats.json");
    
    let new_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "provider": provider,
        "model_name": model_name,
        "duration_seconds": duration_secs,
        "tokens_consumed": total_tokens,
    });

    let mut stats_list = if stats_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&stats_path) {
            serde_json::from_str::<Vec<serde_json::Value>>(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    stats_list.push(new_entry);

    if let Ok(serialized) = serde_json::to_string_pretty(&stats_list) {
        let _ = std::fs::write(&stats_path, serialized);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_categorize_error() {
        assert_eq!(categorize_error(&anyhow!("rate_limited")), ErrorCategory::Transient);
        assert_eq!(categorize_error(&anyhow!("429")), ErrorCategory::Transient);
        assert_eq!(categorize_error(&anyhow!("timeout")), ErrorCategory::Transient);
        assert_eq!(categorize_error(&anyhow!("Fatal database error")), ErrorCategory::Fatal);
    }

    #[test]
    fn test_clean_error_message() {
        assert!(clean_error_message("insufficient_quota").contains("API Quota Exceeded"));
        assert!(clean_error_message("rate_limited").contains("Rate Limit Reached"));
        
        let json_err = r#"{"error": {"message": "Custom error from provider"}}"#;
        assert_eq!(clean_error_message(json_err), "Custom error from provider");

        let adk_err = "Operation failed: error=Something went wrong";
        assert_eq!(clean_error_message(adk_err), "Something went wrong");
    }

    #[tokio::test]
    async fn test_sandbox_security() {
        // These should fail as they attempt to escape via traversal
        assert!(sandbox("../secret.txt").await.is_err());
        
        // Absolute paths outside the allowed directories should fail
        assert!(sandbox("/etc/passwd").await.is_err());
        
        // This should succeed
        let res = sandbox("docs/index.html").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_with_retry() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            "test_op",
            || {
                let c = counter_clone.clone();
                async move {
                    let val = c.fetch_add(1, Ordering::SeqCst);
                    if val < 2 {
                        anyhow::bail!("rate_limited");
                    }
                    Ok("success")
                }
            },
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        ).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
