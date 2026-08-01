use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;
use crate::utils::ignore::NamiIgnore;

pub fn get_http_client() -> &'static reqwest::Client {
    super::client::http_client()
}

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

pub async fn get_workspace_dir() -> std::result::Result<PathBuf, AdkError> {
    if let Ok(env_ws) = std::env::var("NAMI_WORKSPACE") {
        if !env_ws.is_empty() {
            let path = PathBuf::from(env_ws);
            let absolute = clean_unc_path(fs::canonicalize(&path).await.unwrap_or(path));
            return Ok(absolute);
        }
    }

    let current_dir = std::env::current_dir()
        .map_err(|e| AdkError::tool(format!("Failed to get current directory: {}", e)))?;
    let canonical_current = clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir.clone()));

    Ok(canonical_current)
}

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
        if user_path_buf.starts_with(&root) || user_path_buf.starts_with(&nami_dir) || user_path_buf.starts_with(&agents_dir) {
            normalized = user_path_buf;
        } else {
            return Err(AdkError::tool(format!(
                "Security Error: Absolute path '{}' attempts to escape sandbox '{}'.",
                user_path, root.display()
            )));
        }
    } else {
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

    if !normalized.starts_with(&root) && !normalized.starts_with(&nami_dir) && !normalized.starts_with(&agents_dir) {
        return Err(AdkError::tool(format!(
            "Security Error: Path '{}' attempts to escape sandbox.",
            user_path
        )));
    }

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

pub async fn get_km_dir() -> std::result::Result<PathBuf, AdkError> {
    let km_dir = get_nami_dir().join("km");
    if !km_dir.exists() {
        fs::create_dir_all(&km_dir)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create knowledge directory: {}", e)))?;
    }
    Ok(km_dir)
}