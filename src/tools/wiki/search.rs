use crate::utils::get_wiki_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use regex::Regex;
use serde_json::{Value, json};
use std::sync::Arc;
use walkdir::WalkDir;
use super::{
    get_relative_title, ensure_cache_initialized, get_cache,
    SearchWikiArgs, SearchWikiByTagArgs, GlobFindWikiArgs,
};

/// Searches for a keyword across all wiki/OKF concept pages recursively, supporting OKF v0.2 filters.
#[tool]
async fn search_wiki(args: SearchWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut matches = Vec::new();
    let query_lower = args.query.to_lowercase();
    let limit = args.limit.unwrap_or(50);

    let regex_pattern = if args.use_regex.unwrap_or(false) {
        Regex::new(&args.query).ok()
    } else {
        None
    };

    let headers_only = args.headers_only.unwrap_or(false);

    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            if matches.len() >= limit {
                break;
            }

            if let Some(ref req_type) = args.r#type
                && !page.okf.r#type.eq_ignore_ascii_case(req_type)
            {
                continue;
            }

            if let Some(ref req_status) = args.status
                && !page.okf.status.eq_ignore_ascii_case(req_status)
            {
                continue;
            }

            let content = page.content.clone().unwrap_or_else(|| std::fs::read_to_string(&page.path).unwrap_or_default());

            let mut found = false;
            if headers_only {
                for line in content.lines() {
                    if line.starts_with('#') || line.starts_with("---") {
                        if let Some(ref re) = regex_pattern {
                            if re.is_match(line) {
                                found = true;
                                break;
                            }
                        } else if line.to_lowercase().contains(&query_lower) {
                            found = true;
                            break;
                        }
                    }
                }
            } else if let Some(ref re) = regex_pattern {
                if re.is_match(&content) {
                    found = true;
                }
            } else if content.to_lowercase().contains(&query_lower) {
                found = true;
            }

            if found {
                let relative_path = page.path.strip_prefix(&wiki_dir).unwrap_or(&page.path).to_string_lossy().replace("\\", "/");
                matches.push(json!({
                    "title": title,
                    "path": format!("wiki/{}", relative_path),
                    "type": page.okf.r#type,
                    "description": page.okf.description,
                    "status": page.okf.status,
                    "trust_tier": page.trust_tier,
                    "is_stale": page.is_stale,
                }));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": "No matches found in wiki/OKF concepts." }))
    } else {
        Ok(json!({ "matches": matches, "limit_applied": limit }))
    }
}

/// Searches all wiki pages for a specific tag recursively from the Cache.
#[tool]
async fn search_wiki_by_tag(args: SearchWikiByTagArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut matches = Vec::new();
    let target_tag = args.tag.to_lowercase();

    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            if page.tags.iter().any(|t| t.to_lowercase() == target_tag) {
                let relative_path = page.path.strip_prefix(&wiki_dir).unwrap_or(&page.path).to_string_lossy().replace("\\", "/");
                matches.push(json!({
                    "title": title,
                    "path": format!("wiki/{}", relative_path)
                }));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": format!("No pages found with tag '#{}'.", args.tag) }))
    } else {
        Ok(json!({ "tag": args.tag, "matches": matches }))
    }
}

/// Finds wiki pages matching glob pattern.
#[tool]
async fn glob_find_wiki(args: GlobFindWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let glob = globset::Glob::new(&args.pattern)
        .map_err(|e| AdkError::tool(format!("Invalid glob pattern: {}", e)))?
        .compile_matcher();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative = path.strip_prefix(&wiki_dir).unwrap_or(path);
            let rel_str = relative.to_string_lossy().replace("\\", "/");
            if glob.is_match(&rel_str) {
                matches.push(json!({
                    "title": get_relative_title(&wiki_dir, path),
                    "path": format!("wiki/{}", rel_str)
                }));
            }
        }
    }

    Ok(json!({ "matches": matches }))
}

pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(SearchWiki),
        Arc::new(SearchWikiByTag),
        Arc::new(GlobFindWiki),
    ]
}
