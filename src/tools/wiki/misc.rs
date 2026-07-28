use crate::utils::get_wiki_dir;
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
    get_cache, git_auto_commit, SummarizeWikiArgs, SanitizeWikiVaultArgs,
};

/// Generates a 'SUMMARY.md' file indexing all pages recursively from the Cache.
#[tool]
async fn summarize_wiki(_args: SummarizeWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut pages_to_read = Vec::new();
    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            if title == "SUMMARY" {
                continue;
            }
            pages_to_read.push((
                title.clone(),
                page.path.clone(),
                page.frontmatter.get("title").cloned().unwrap_or_else(|| title.clone()),
                page.frontmatter.get("description").cloned()
            ));
        }
    }

    let mut pages_info = Vec::new();
    for (title, path, display_title, description_opt) in pages_to_read {
        let content = fs::read_to_string(&path).await.unwrap_or_default();
        let mut description = description_opt.unwrap_or_else(|| "No description available.".to_string());

        if description == "No description available." {
            let first_line = content
                .lines()
                .skip_while(|l| l.starts_with("---") || l.trim().is_empty())
                .next()
                .unwrap_or("No content")
                .trim_start_matches('#')
                .trim();
            if !first_line.is_empty() {
                description = first_line.to_string();
            }
        }

        pages_info.push((title, display_title, description));
    }

    pages_info.sort_by(|a, b| a.0.cmp(&b.0));

    let mut summary_content = "# Wiki Summary Index\n\nGenerated automatically by Nami.\n\n".to_string();
    for (path, display, desc) in pages_info {
        summary_content.push_str(&format!("- **[{}]({})**: {}\n", display, path, desc));
    }

    let summary_path = wiki_dir.join("SUMMARY.md");
    fs::write(&summary_path, &summary_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write SUMMARY.md: {}", e)))?;

    Ok(json!({"status": "success", "message": "Wiki summary (SUMMARY.md) has been updated!"}))
}

/// Sanitizes wiki vault titles and pages from Cache.
#[tool]
async fn sanitize_wiki_vault(_args: SanitizeWikiVaultArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut files_to_process = Vec::new();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&wiki_dir, path);
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
        let old_path = wiki_dir.join(format!("{}.md", old_title));
        let new_path = wiki_dir.join(format!("{}.md", new_title));

        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path).await.map_err(|e| {
                AdkError::tool(format!("Failed to rename {:?} to {:?}: {}", old_path, new_path, e))
            })?;
            renamed_count += 1;
            git_auto_commit(&wiki_dir, &old_path, &format!("wiki: sanitize rename delete {}", old_title));
            git_auto_commit(&wiki_dir, &new_path, &format!("wiki: sanitize rename create {}", new_title));
        }
    }

    let current_files: Vec<PathBuf> = files_to_process
        .into_iter()
        .map(|p| {
            let relative = get_relative_title(&wiki_dir, &p);
            if let Some(new_rel) = rename_map.get(&relative) {
                wiki_dir.join(format!("{}.md", new_rel))
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
                git_auto_commit(&wiki_dir, &path, "wiki: sanitize update links");
            }
        }
    }

    // Reset cache
    if let Ok(mut cache) = get_cache().write() {
        cache.initialized = false;
    }
    let _ = ensure_cache_initialized(&wiki_dir).await;

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
        Arc::new(SummarizeWiki),
        Arc::new(SanitizeWikiVault),
    ]
}
