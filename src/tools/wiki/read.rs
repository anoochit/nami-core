use crate::utils::get_wiki_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::fs;
use super::{WikiPageArgs, ensure_cache_initialized, get_cache};

#[tool]
async fn get_wiki_page(args: WikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = super::sanitize_title(&args.title);
    let filename = format!("{}.md", sanitized_title);
    let path = wiki_dir.join(&filename);

    if !path.exists() {
        return Err(AdkError::tool(format!("Wiki page '{}' not found.", args.title)));
    }

    ensure_cache_initialized(&wiki_dir).await?;

    let cached_content = {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        cache.pages.get(&sanitized_title).and_then(|page| page.content.clone())
    };

    let full_content = match cached_content {
        Some(content) => content,
        None => fs::read_to_string(&path)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to read wiki page: {}", e)))?,
    };

    let lines: Vec<&str> = full_content.lines().collect();
    let total_lines = lines.len();

    let start_line = args.start_line.unwrap_or(1).max(1);
    let end_line = args.end_line.unwrap_or(total_lines).min(total_lines);

    if start_line > total_lines {
        return Err(AdkError::tool(format!(
            "Requested start_line ({}) is greater than total lines ({})",
            start_line, total_lines
        )));
    }

    let final_end = if end_line - start_line + 1 > 800 {
        start_line + 799
    } else {
        end_line
    };

    let paged_lines = &lines[start_line - 1..final_end];
    let content = paged_lines.join("\n");
    let is_truncated = final_end < total_lines;

    Ok(json!({
        "title": args.title,
        "content": content,
        "path": format!("wiki/{}", filename),
        "start_line": start_line,
        "end_line": final_end,
        "total_lines": total_lines,
        "is_truncated": is_truncated,
        "message": if is_truncated {
            format!("Notice: Content was truncated. Read lines {}-{}. Pass parameters 'start_line' and 'end_line' to fetch more.", start_line, final_end)
        } else {
            "".to_string()
        }
    }))
}

pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetWikiPage)]
}
