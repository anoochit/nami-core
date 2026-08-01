use crate::utils::get_km_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;
use super::{
    get_relative_title, to_title_case, ensure_cache_initialized,
    get_cache, git_auto_commit, SummarizeKmArgs, SanitizeKmVaultArgs,
};

/// Generates 'index.md' (and 'SUMMARY.md') indexing all concept pages recursively per OKF v0.2 §8.
#[tool]
async fn summarize_km(_args: SummarizeKmArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    ensure_cache_initialized(&km_dir).await?;

    let mut pages_to_read = Vec::new();
    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            if title == "SUMMARY" || title == "index" || title == "log" {
                continue;
            }
            let concept_type = page.okf.r#type.clone();
            pages_to_read.push((
                title.clone(),
                page.path.clone(),
                page.okf.title.clone().unwrap_or_else(|| title.clone()),
                page.okf.description.clone(),
                concept_type,
            ));
        }
    }

    let mut pages_info = Vec::new();
    for (title, path, display_title, description_opt, concept_type) in pages_to_read {
        let content = fs::read_to_string(&path).await.unwrap_or_default();
        let mut description = description_opt.unwrap_or_else(|| "No description available.".to_string());

        if description == "No description available." {
            let first_line = content
                .lines()
                .find(|l| !l.starts_with("---") && !l.trim().is_empty())
                .unwrap_or("No content")
                .trim_start_matches('#')
                .trim();
            if !first_line.is_empty() {
                description = first_line.to_string();
            }
        }

        pages_info.push((title, display_title, description, concept_type));
    }

    pages_info.sort_by(|a, b| a.0.cmp(&b.0));

    let mut index_content = "---\nokf_version: \"0.2\"\n---\n\n# Knowledge Index\n\nGenerated automatically by Nami per Open Knowledge Format (OKF v0.2).\n\n".to_string();
    for (path, display, desc, ctype) in pages_info {
        index_content.push_str(&format!("- **[{}]({})** (`{}`): {}\n", display, path, ctype, desc));
    }

    let index_path = km_dir.join("index.md");
    fs::write(&index_path, &index_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write index.md: {}", e)))?;

    let summary_path = km_dir.join("SUMMARY.md");
    fs::write(&summary_path, &index_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write SUMMARY.md: {}", e)))?;

    Ok(json!({"status": "success", "message": "Knowledge Index (index.md & SUMMARY.md) updated per OKF v0.2!"}))
}

/// Sanitizes knowledge vault titles and pages from Cache.
#[tool]
async fn sanitize_km_vault(_args: SanitizeKmVaultArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut files_to_process = Vec::new();

    for entry in WalkDir::new(&km_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&km_dir, path);
            files_to_process.push(path.to_path_buf());

            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let new_stem = to_title_case(&file_stem);

            if file_stem != new_stem {
                let mut new_title = relative_title.clone();
                if let Some(pos) = new_title.rfind(&file_stem) {
                    new_title.replace_range(pos..pos + file_stem.len(), &new_stem);
                }
                rename_map.insert(relative_title, new_title);
            }
        }
    }

    let mut renamed_count = 0;
    let mut links_updated_count = 0;

    for (old_title, new_title) in &rename_map {
        let old_path = km_dir.join(format!("{}.md", old_title));
        let new_path = km_dir.join(format!("{}.md", new_title));

        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path).await.map_err(|e| {
                AdkError::tool(format!("Failed to rename {:?} to {:?}: {}", old_path, new_path, e))
            })?;
            renamed_count += 1;
            git_auto_commit(&km_dir, &old_path, &format!("km: sanitize rename delete {}", old_title));
            git_auto_commit(&km_dir, &new_path, &format!("km: sanitize rename create {}", new_title));
        }
    }

    let current_files: Vec<PathBuf> = files_to_process
        .into_iter()
        .map(|p| {
            let relative = get_relative_title(&km_dir, &p);
            if let Some(new_rel) = rename_map.get(&relative) {
                km_dir.join(format!("{}.md", new_rel))
            } else {
                p
            }
        })
        .collect();

    for path in current_files {
        if path.exists() {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            let mut new_content = content.clone();

            for (old_title, new_title) in &rename_map {
                let old_link = format!("[[{}]]", old_title);
                let new_link = format!("[[{}]]", new_title);
                if new_content.contains(&old_link) {
                    new_content = new_content.replace(&old_link, &new_link);
                    links_updated_count += 1;
                }
            }

            if content != new_content {
                fs::write(&path, new_content).await.map_err(|e| {
                    AdkError::tool(format!("Failed to update links in {:?}: {}", path, e))
                })?;
                git_auto_commit(&km_dir, &path, "km: sanitize update links");
            }
        }
    }

    // Reset cache
    if let Ok(mut cache) = get_cache().write() {
        cache.initialized = false;
    }
    let _ = ensure_cache_initialized(&km_dir).await;

    Ok(json!({
        "status": "success",
        "message": "Vault cleanup complete.",
        "files_renamed": renamed_count,
        "links_updated": links_updated_count,
        "renamed_mapping": rename_map
    }))
}

pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(SummarizeKm),
        Arc::new(SanitizeKmVault),
    ]
}
