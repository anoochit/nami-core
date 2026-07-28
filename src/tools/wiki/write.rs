use crate::utils::get_wiki_dir;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use chrono::{Datelike, Utc};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;
use super::{
    sanitize_title, parse_wiki_file_sync, get_cache, git_auto_commit,
    expand_template_variables, ensure_cache_initialized,
    AddWikiArgs, WikiPageArgs, CreateDailyNoteArgs,
    ApplyTemplateArgs, RenameWikiPageArgs,
};

/// Adds or updates a wiki page in the 'wiki/' directory. Supports nested folders.
#[tool]
async fn add_wiki_page(args: AddWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = sanitize_title(&args.title);
    let filename = format!("{}.md", sanitized_title);
    let path = wiki_dir.join(&filename);

    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create parent directories: {}", e)))?;
    }

    let is_append = args.append.unwrap_or(false) && path.exists();
    if is_append {
        let mut existing = fs::read_to_string(&path).await.unwrap_or_default();
        existing.push_str("\n\n");
        existing.push_str(&args.content);
        fs::write(&path, existing)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to append to wiki page: {}", e)))?;
    } else {
        let mut final_content = args.content.trim().to_string();

        if !final_content.starts_with("---") {
            let today = Utc::now();
            let date_str = format!("{}-{:02}-{:02}", today.year(), today.month(), today.day());
            let title_basename = Path::new(&sanitized_title)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();

            let frontmatter = format!(
                "---\ntitle: {}\ndate: {}\ntags: []\n---\n\n",
                title_basename, date_str
            );

            if !final_content.starts_with('#') && !final_content.is_empty() {
                final_content = format!("{}# {}\n\n{}", frontmatter, title_basename, final_content);
            } else if final_content.is_empty() {
                final_content = format!("{}# {}\n\n", frontmatter, title_basename);
            } else {
                final_content = format!("{}{}", frontmatter, final_content);
            }
        }

        fs::write(&path, &final_content)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to write wiki page: {}", e)))?;
    }

    // Parse and update cache synchronously to avoid locks across awaits
    if let Ok(metadata) = parse_wiki_file_sync(&wiki_dir, &path) {
        if let Ok(mut cache) = get_cache().write() {
            cache.pages.insert(sanitized_title.clone(), metadata);
        }
    }

    // Git auto commit
    let action_msg = if is_append {
        format!("wiki: append to {}", sanitized_title)
    } else {
        format!("wiki: update {}", sanitized_title)
    };
    git_auto_commit(&wiki_dir, &path, &action_msg);

    Ok(json!({
        "status": "success",
        "message": format!("Saved wiki page '{}'", args.title),
        "path": format!("wiki/{}", filename)
    }))
}

/// Creates a new wiki page for the current date with dynamic variable expansion.
#[tool]
async fn create_daily_note(args: CreateDailyNoteArgs) -> std::result::Result<Value, AdkError> {
    let today = Utc::now();
    let title = format!(
        "Daily Notes/{}-{:02}-{:02}",
        today.year(),
        today.month(),
        today.day()
    );

    let mut final_content = args.content.unwrap_or_else(|| format!("# {}\n\n", title));

    if let Some(template_name) = args.template {
        let wiki_dir = get_wiki_dir().await?;
        let template_path = wiki_dir.join("Templates").join(format!("{}.md", template_name));
        if template_path.exists() {
            let template_content = fs::read_to_string(&template_path).await.unwrap_or_default();
            final_content = expand_template_variables(&template_content, &title);
        }
    }

    let add_args = AddWikiArgs {
        title: title.clone(),
        content: final_content,
        append: Some(false),
    };

    add_wiki_page(add_args).await?;

    Ok(json!({"status": "success", "message": format!("Created daily note for '{}'", title)}))
}

/// Applies a template to a page expanding dynamic placeholders.
#[tool]
async fn apply_template(args: ApplyTemplateArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let template_path = wiki_dir.join("Templates").join(format!("{}.md", args.template_name));

    if !template_path.exists() {
        return Err(AdkError::tool(format!("Template '{}' not found in Templates folder.", args.template_name)));
    }

    let template_content = fs::read_to_string(&template_path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read template: {}", e)))?;

    let title_basename = Path::new(&args.title).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let final_content = expand_template_variables(&template_content, &title_basename);

    let add_args = AddWikiArgs {
        title: args.title.clone(),
        content: final_content,
        append: Some(false),
    };

    add_wiki_page(add_args).await?;

    Ok(json!({
        "status": "success",
        "message": format!("Applied template '{}' to '{}'", args.template_name, args.title)
    }))
}

/// Renames a wiki page updating cache and executing silent Git commits.
#[tool]
async fn rename_wiki_page(args: RenameWikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let old_sanitized = sanitize_title(&args.old_title);
    let new_sanitized = sanitize_title(&args.new_title);

    let old_path = wiki_dir.join(format!("{}.md", old_sanitized));
    let new_path = wiki_dir.join(format!("{}.md", new_sanitized));

    if !old_path.exists() {
        return Err(AdkError::tool(format!("Wiki page '{}' not found.", args.old_title)));
    }

    if new_path.exists() {
        return Err(AdkError::tool(format!("Destination page '{}' already exists.", args.new_title)));
    }

    if let Some(parent) = new_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create parent directories: {}", e)))?;
    }

    fs::rename(&old_path, &new_path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to rename file: {}", e)))?;

    // Update links in all files
    let mut links_updated = 0;
    let old_link_exact = format!("[[{}]]", args.old_title);
    let old_link_sanitized = format!("[[{}]]", old_sanitized);
    let old_filename = old_path.file_stem().unwrap_or_default().to_string_lossy();
    let old_link_short = format!("[[{}]]", old_filename);
    let new_link = format!("[[{}]]", args.new_title);

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            let mut new_content = content.clone();

            if new_content.contains(&old_link_exact) {
                new_content = new_content.replace(&old_link_exact, &new_link);
            }
            if new_content.contains(&old_link_sanitized) && old_link_exact != old_link_sanitized {
                new_content = new_content.replace(&old_link_sanitized, &new_link);
            }
            if new_content.contains(&old_link_short)
                && old_link_exact != old_link_short
                && old_link_sanitized != old_link_short
            {
                new_content = new_content.replace(&old_link_short, &new_link);
            }

            if content != new_content {
                fs::write(&path, new_content).await.map_err(|e| {
                    AdkError::tool(format!("Failed to update links in {:?}: {}", path, e))
                })?;
                links_updated += 1;
            }
        }
    }

    // Refresh cache completely
    if let Ok(mut cache) = get_cache().write() {
        cache.initialized = false;
    }
    let _ = ensure_cache_initialized(&wiki_dir).await;

    // Git commits
    git_auto_commit(&wiki_dir, &old_path, &format!("wiki: rename delete {}", old_sanitized));
    git_auto_commit(&wiki_dir, &new_path, &format!("wiki: rename create {}", new_sanitized));

    Ok(json!({
        "status": "success",
        "message": format!("Renamed '{}' to '{}'.", args.old_title, args.new_title),
        "files_updated_with_new_links": links_updated
    }))
}

/// Deletes a wiki page from the directory, resetting cache and commiting.
#[tool]
async fn delete_wiki_page(args: WikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = sanitize_title(&args.title);
    let filename = format!("{}.md", sanitized_title);
    let path = wiki_dir.join(&filename);

    if !path.exists() {
        return Err(AdkError::tool(format!("Wiki page '{}' not found.", args.title)));
    }

    fs::remove_file(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to delete wiki page: {}", e)))?;

    if let Ok(mut cache) = get_cache().write() {
        cache.pages.remove(&sanitized_title);
    }

    git_auto_commit(&wiki_dir, &path, &format!("wiki: delete {}", sanitized_title));

    Ok(json!({
        "status": "success",
        "message": format!("Successfully deleted wiki page '{}'.", args.title),
        "path": format!("wiki/{}", filename)
    }))
}

pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AddWikiPage),
        Arc::new(CreateDailyNote),
        Arc::new(ApplyTemplate),
        Arc::new(RenameWikiPage),
        Arc::new(DeleteWikiPage),
    ]
}
