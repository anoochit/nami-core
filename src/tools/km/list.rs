use crate::utils::get_km_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use super::{
    sanitize_title, ensure_cache_initialized, get_cache,
    ListKmPagesArgs, GetKmGraphArgs, GetBacklinksArgs, CheckBrokenLinksArgs,
};

/// Lists all available knowledge pages recursively from Cache, supporting OKF v0.2 filtering.
#[tool]
async fn list_km_pages(args: ListKmPagesArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    ensure_cache_initialized(&km_dir).await?;

    let mut pages = Vec::new();
    {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        for (title, page) in &cache.pages {
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

            let relative_path = page.path.strip_prefix(&km_dir).unwrap_or(&page.path).to_string_lossy().replace("\\", "/");
            pages.push(json!({
                "title": title,
                "path": format!("km/{}", relative_path),
                "type": page.okf.r#type,
                "description": page.okf.description,
                "status": page.okf.status,
                "trust_tier": page.trust_tier,
                "is_stale": page.is_stale,
                "tags": page.tags
            }));
        }
    }

    pages.sort_by(|a, b| a["title"].as_str().unwrap_or_default().cmp(b["title"].as_str().unwrap_or_default()));

    Ok(json!({ "pages": pages }))
}

/// Scans all knowledge pages recursively for backlinks from Cache to build a knowledge graph.
#[tool]
async fn get_km_graph(_args: GetKmGraphArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    ensure_cache_initialized(&km_dir).await?;

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

/// Finds all knowledge pages linking to the specified target using the Cache.
#[tool]
async fn get_backlinks(args: GetBacklinksArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    ensure_cache_initialized(&km_dir).await?;

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

/// Scans all knowledge pages for wikilinks pointing to non-existent pages using the Cache.
#[tool]
async fn check_broken_links(_args: CheckBrokenLinksArgs) -> std::result::Result<Value, AdkError> {
    let km_dir = get_km_dir().await?;
    ensure_cache_initialized(&km_dir).await?;

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
        Arc::new(ListKmPages),
        Arc::new(GetKmGraph),
        Arc::new(GetBacklinks),
        Arc::new(CheckBrokenLinks),
    ]
}
