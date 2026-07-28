use crate::utils::get_wiki_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use super::{
    sanitize_title, ensure_cache_initialized, get_cache,
    ListWikiPagesArgs, GetWikiGraphArgs, GetBacklinksArgs, CheckBrokenLinksArgs,
};

/// Lists all available wiki pages recursively from Cache.
#[tool]
async fn list_wiki_pages(_args: ListWikiPagesArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut pages = Vec::new();
    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            let relative_path = page.path.strip_prefix(&wiki_dir).unwrap_or(&page.path).to_string_lossy().replace("\\", "/");
            pages.push(json!({
                "title": title,
                "path": format!("wiki/{}", relative_path),
                "tags": page.tags
            }));
        }
    }

    pages.sort_by(|a, b| a["title"].as_str().unwrap_or_default().cmp(b["title"].as_str().unwrap_or_default()));

    Ok(json!({ "pages": pages }))
}

/// Scans all wiki pages recursively for backlinks from Cache to build a knowledge graph.
#[tool]
async fn get_wiki_graph(_args: GetWikiGraphArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            nodes.push(json!({"id": title, "label": title}));
            for target in &page.links {
                edges.push(json!({"source": title, "target": target}));
            }
        }
    }

    Ok(json!({ "nodes": nodes, "edges": edges }))
}

/// Finds all wiki pages linking to the specified target using the Cache.
#[tool]
async fn get_backlinks(args: GetBacklinksArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut backlinks = Vec::new();
    let target_title = sanitize_title(&args.title);

    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
            if page.links.contains(&target_title) {
                backlinks.push(title.clone());
            }
        }
    }

    if backlinks.is_empty() {
        Ok(json!({ "message": format!("No backlinks found for '{}'", args.title) }))
    } else {
        Ok(json!({ "target": args.title, "backlinks": backlinks }))
    }
}

/// Scans all wiki pages for wikilinks pointing to non-existent pages using the Cache.
#[tool]
async fn check_broken_links(_args: CheckBrokenLinksArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    ensure_cache_initialized(&wiki_dir).await?;

    let mut broken_links = HashMap::new();
    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        let all_page_keys_lower: Vec<String> = cache.pages.keys().map(|k| k.to_lowercase()).collect();

        for (title, page) in &cache.pages {
            for target in &page.links {
                let target_lower = target.to_lowercase();
                let exists = if target.contains('/') {
                    all_page_keys_lower.contains(&target_lower)
                } else {
                    all_page_keys_lower.iter().any(|k| k.ends_with(&target_lower))
                };

                if !exists {
                    broken_links.entry(target.clone()).or_insert_with(Vec::new).push(title.clone());
                }
            }
        }
    }

    if broken_links.is_empty() {
        Ok(json!({ "message": "No broken links found." }))
    } else {
        Ok(json!({ "broken_links": broken_links }))
    }
}

pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ListWikiPages),
        Arc::new(GetWikiGraph),
        Arc::new(GetBacklinks),
        Arc::new(CheckBrokenLinks),
    ]
}
