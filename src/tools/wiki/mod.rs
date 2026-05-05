use crate::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use chrono::{Datelike, Utc};
use regex::Regex;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Deserialize, JsonSchema)]
struct WikiPageArgs {
    /// The title of the wiki page (e.g., 'project-notes'). This will be used as the filename.
    title: String,
}

#[derive(Deserialize, JsonSchema)]
struct AddWikiArgs {
    /// The title of the wiki page.
    title: String,
    /// The content in Markdown format.
    content: String,
    /// If true, appends to the existing page instead of overwriting.
    append: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchWikiArgs {
    /// The keyword or phrase to search for across all wiki pages.
    query: String,
    /// Optional: If true, treats the query as a Regular Expression.
    use_regex: Option<bool>,
    /// Optional: If true, searches only within YAML frontmatter and Markdown headers.
    headers_only: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchWikiByTagArgs {
    /// The tag to search for (e.g., 'rust', 'project-ideas'). Do not include the '#' symbol.
    tag: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetWikiGraphArgs {}

#[derive(Deserialize, JsonSchema)]
struct CreateDailyNoteArgs {
    /// Optional content to pre-fill the daily note with.
    content: Option<String>,
    /// Optional template name to use from the 'Templates' folder (e.g., 'DailyTemplate').
    template: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SanitizeWikiVaultArgs {}

#[derive(Deserialize, JsonSchema)]
struct GetBacklinksArgs {
    /// The title of the wiki page to find backlinks for.
    title: String,
}

#[derive(Deserialize, JsonSchema)]
struct CheckBrokenLinksArgs {}

#[derive(Deserialize, JsonSchema)]
struct RenameWikiPageArgs {
    /// The current title of the wiki page.
    old_title: String,
    /// The new title for the wiki page.
    new_title: String,
}

#[derive(Deserialize, JsonSchema)]
struct ApplyTemplateArgs {
    /// The title of the wiki page to create or overwrite.
    title: String,
    /// The name of the template file in the 'Templates' folder (without .md extension).
    template_name: String,
}

/// Helper to get the wiki directory path.
async fn get_wiki_dir() -> std::result::Result<PathBuf, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;
    let wiki_dir = root.join("wiki");
    if !wiki_dir.exists() {
        fs::create_dir_all(&wiki_dir)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create wiki directory: {}", e)))?;
    }
    Ok(wiki_dir)
}

/// Helper to convert a string to Title Case (Obsidian-style with spaces).
/// Converts dashes (except in dates), underscores, and multiple spaces into single spaces and capitalizes each word.
fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    let mut last_was_space = false;
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];

        // Special case: Preserve dashes if they look like a date (Digit-Dash-Digit)
        let is_date_dash = c == '-'
            && i > 0
            && i < chars.len() - 1
            && chars[i - 1].is_ascii_digit()
            && chars[i + 1].is_ascii_digit();

        if (c == '-' && !is_date_dash) || c == '_' || c == ' ' {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
            last_was_space = false;
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result.trim().to_string()
}

/// Helper to sanitize title into a filename while enforcing Obsidian style (Title Case with spaces).
/// Removes characters that are invalid in Windows/Unix filenames, except for forward slashes.
fn sanitize_title(title: &str) -> String {
    // Convert backslashes to forward slashes for cross-platform folder support
    let sanitized_path = title.trim().replace("\\", "/");

    // Process each component of the path separately to preserve folder structure
    let parts: Vec<String> = sanitized_path
        .split('/')
        .map(|part| {
            let mut p = part.to_string();
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            p.retain(|c| !invalid_chars.contains(&c));
            to_title_case(&p)
        })
        .collect();

    parts.join("/")
}

/// Helper to get the relative title from a file path
fn get_relative_title(wiki_dir: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(wiki_dir)
        .unwrap_or(file_path)
        .with_extension("")
        .to_string_lossy()
        .replace("\\", "/") // Normalize to forward slashes for cross-platform consistency
}

/// Adds or updates a wiki page in the 'wiki/' directory. Supports nested folders (e.g., 'Projects/Notes').
#[tool]
async fn add_wiki_page(args: AddWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = sanitize_title(&args.title);
    let filename = format!("{}.md", sanitized_title);
    let path = wiki_dir.join(&filename);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent()
        && !parent.exists() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AdkError::tool(format!("Failed to create parent directories: {}", e))
            })?;
        }

    if args.append.unwrap_or(false) && path.exists() {
        let mut existing = fs::read_to_string(&path).await.unwrap_or_default();
        existing.push_str("\n\n");
        existing.push_str(&args.content);
        fs::write(&path, existing)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to append to wiki page: {}", e)))?;
        Ok(
            json!({"status": "success", "message": format!("Appended to wiki page '{}'", args.title)}),
        )
    } else {
        let mut final_content = args.content.trim().to_string();

        // If it's a new file (or overwrite) and doesn't start with frontmatter, add it.
        // We also ensure there's a level-1 header.
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
        Ok(json!({"status": "success", "message": format!("Saved wiki page '{}'", args.title)}))
    }
}

/// Retrieves the content of a specific wiki page.
#[tool]
async fn get_wiki_page(args: WikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let filename = format!("{}.md", sanitize_title(&args.title));
    let path = wiki_dir.join(filename);

    if !path.exists() {
        return Err(AdkError::tool(format!(
            "Wiki page '{}' not found.",
            args.title
        )));
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read wiki page: {}", e)))?;

    Ok(json!({ "title": args.title, "content": content }))
}

/// Lists all available wiki pages recursively.
#[tool]
async fn list_wiki_pages(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut pages = Vec::new();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            pages.push(get_relative_title(&wiki_dir, path));
        }
    }

    Ok(json!({ "pages": pages }))
}

/// Searches for a keyword across all wiki pages recursively.
#[tool]
async fn search_wiki(args: SearchWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let query_lower = args.query.to_lowercase();

    let regex_pattern = if args.use_regex.unwrap_or(false) {
        Regex::new(&args.query).ok()
    } else {
        None
    };

    let headers_only = args.headers_only.unwrap_or(false);

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();

            let mut found = false;
            if headers_only {
                // Simple check for Markdown headers or YAML blocks
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
                    } else if found || (line.trim().is_empty() && !content.starts_with("---")) {
                        // Stop checking if we leave the frontmatter/header area early on
                        // This is a naive approach, a full YAML parser would be better for strictly frontmatter
                    }
                }
            } else {
                if let Some(ref re) = regex_pattern {
                    if re.is_match(&content) {
                        found = true;
                    }
                } else if content.to_lowercase().contains(&query_lower) {
                    found = true;
                }
            }

            if found {
                matches.push(get_relative_title(&wiki_dir, path));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": "No matches found in wiki." }))
    } else {
        Ok(json!({ "matches": matches }))
    }
}

/// Searches all wiki pages for a specific tag (e.g., '#rust') recursively.
#[tool]
async fn search_wiki_by_tag(args: SearchWikiByTagArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let tag_pattern = format!("#{}", args.tag.to_lowercase());

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();

            // Look for tags in frontmatter or in content
            if content.to_lowercase().contains(&tag_pattern) {
                matches.push(get_relative_title(&wiki_dir, path));
                continue;
            }

            // Simple frontmatter tag check (naive)
            if content.starts_with("---") {
                let end_idx = content[3..].find("---").unwrap_or(0);
                if end_idx > 0 {
                    let frontmatter = &content[3..end_idx + 3];
                    if frontmatter
                        .to_lowercase()
                        .contains(&format!("tags: {}", args.tag.to_lowercase()))
                        || frontmatter
                            .to_lowercase()
                            .contains(&format!("- {}", args.tag.to_lowercase()))
                    {
                        matches.push(get_relative_title(&wiki_dir, path));
                    }
                }
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": format!("No pages found with tag \'#{}\'.", args.tag) }))
    } else {
        Ok(json!({ "tag": args.tag, "matches": matches }))
    }
}

/// Scans all wiki pages recursively for [[wikilink]] references and builds a knowledge graph.
#[tool]
async fn get_wiki_graph(_args: GetWikiGraphArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let wikilink_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let title = get_relative_title(&wiki_dir, path);
            nodes.push(json!({"id": &title, "label": &title}));

            let content = fs::read_to_string(&path).await.unwrap_or_default();
            for cap in wikilink_regex.captures_iter(&content) {
                let target_title = cap[1].to_string();
                edges.push(json!({"source": &title, "target": &target_title}));
            }
        }
    }

    Ok(json!({ "nodes": nodes, "edges": edges }))
}

/// Creates a new wiki page for the current date (e.g., 'Daily Notes/YYYY-MM-DD.md').
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
        let template_path = wiki_dir
            .join("Templates")
            .join(format!("{}.md", template_name));
        if template_path.exists() {
            let template_content = fs::read_to_string(&template_path).await.unwrap_or_default();
            // Simple template replacement
            final_content = template_content
                .replace(
                    "{{date}}",
                    &format!("{}-{:02}-{:02}", today.year(), today.month(), today.day()),
                )
                .replace("{{title}}", &title);
        }
    }

    let add_args = AddWikiArgs {
        title: title.clone(),
        content: final_content,
        append: Some(false),
    };

    add_wiki_page(add_args).await?;

    Ok(
        json!({"status": "success", "message": format!("Created daily note for '{}'", title), "title": title}),
    )
}

/// Generates a 'SUMMARY.md' file that indexes all available wiki pages recursively with a brief overview.
#[tool]
async fn summarize_wiki(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut summary_content =
        "# Wiki Summary Index\n\nGenerated automatically by Nami.\n\n".to_string();

    // Helper to parse simple YAML frontmatter for title/description
    let mut pages_info = Vec::new();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&wiki_dir, path);
            if relative_title == "SUMMARY" {
                continue;
            }

            let content = fs::read_to_string(&path).await.unwrap_or_default();
            let mut display_title = relative_title.clone();
            let mut description = "No description available.".to_string();

            // Very basic YAML frontmatter parsing just for the summary
            if content.starts_with("---\n")
                && let Some(end_idx) = content[4..].find("\n---\n") {
                    let frontmatter = &content[4..end_idx + 4];
                    if let Ok(docs) = yaml_rust::YamlLoader::load_from_str(frontmatter)
                        && !docs.is_empty() {
                            let doc = &docs[0];
                            if let Some(t) = doc["title"].as_str() {
                                display_title = t.to_string();
                            }
                            if let Some(d) = doc["description"].as_str() {
                                description = d.to_string();
                            }
                        }
                }

            if description == "No description available." {
                // Fallback to first line
                let first_line = content
                    .lines()
                    .skip_while(|l| l.starts_with("---") || l.trim().is_empty()) // skip frontmatter
                    .next()
                    .unwrap_or("No content")
                    .trim_start_matches('#')
                    .trim();
                if !first_line.is_empty() {
                    description = first_line.to_string();
                }
            }

            pages_info.push((relative_title, display_title, description));
        }
    }

    pages_info.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by path

    for (path, display, desc) in pages_info {
        summary_content.push_str(&format!("- **[{}]({})**: {}\n", display, path, desc));
    }

    let summary_path = wiki_dir.join("SUMMARY.md");
    fs::write(&summary_path, summary_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write SUMMARY.md: {}", e)))?;

    Ok(json!({"status": "success", "message": "Wiki summary (SUMMARY.md) has been updated!"}))
}

/// Applies a template to a new or existing wiki page.
#[tool]
async fn apply_template(args: ApplyTemplateArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let template_path = wiki_dir
        .join("Templates")
        .join(format!("{}.md", args.template_name));

    if !template_path.exists() {
        return Err(AdkError::tool(format!(
            "Template '{}' not found in Templates folder.",
            args.template_name
        )));
    }

    let template_content = fs::read_to_string(&template_path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read template: {}", e)))?;

    let today = Utc::now();
    let title_basename = Path::new(&args.title)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    // Simple variable replacement
    let final_content = template_content
        .replace(
            "{{date}}",
            &format!("{}-{:02}-{:02}", today.year(), today.month(), today.day()),
        )
        .replace("{{title}}", &title_basename);

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

/// Finds all wiki pages that contain a wikilink [[target]] to the specified title.
#[tool]
async fn get_backlinks(args: GetBacklinksArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut backlinks = Vec::new();
    let target_link = format!("[[{}]]", args.title);

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            if content.contains(&target_link) {
                backlinks.push(get_relative_title(&wiki_dir, path));
            }
        }
    }

    if backlinks.is_empty() {
        Ok(json!({ "message": format!("No backlinks found for '{}'", args.title) }))
    } else {
        Ok(json!({ "target": args.title, "backlinks": backlinks }))
    }
}

/// Scans all wiki pages for wikilinks that point to non-existent pages.
#[tool]
async fn check_broken_links(_args: CheckBrokenLinksArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut all_pages = Vec::new();
    let mut broken_links = HashMap::new();
    let wikilink_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    // Pass 1: Collect all existing page titles
    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            all_pages.push(get_relative_title(&wiki_dir, path).to_lowercase());
        }
    }

    // Pass 2: Check links in all pages
    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let title = get_relative_title(&wiki_dir, path);
            let content = fs::read_to_string(&path).await.unwrap_or_default();

            for cap in wikilink_regex.captures_iter(&content) {
                let target_title = cap[1].to_string();
                let sanitized_target = sanitize_title(&target_title).to_lowercase();

                // If the link contains a slash, check the exact sanitized path, otherwise check if any file ends with this name
                let exists = if target_title.contains('/') {
                    all_pages.contains(&sanitized_target)
                } else {
                    all_pages.iter().any(|p| p.ends_with(&sanitized_target))
                };

                if !exists {
                    broken_links
                        .entry(target_title)
                        .or_insert_with(Vec::new)
                        .push(title.clone());
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

/// Renames a wiki page and updates all wikilinks pointing to it across the vault.
#[tool]
async fn rename_wiki_page(args: RenameWikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let old_sanitized = sanitize_title(&args.old_title);
    let new_sanitized = sanitize_title(&args.new_title);

    let old_path = wiki_dir.join(format!("{}.md", old_sanitized));
    let new_path = wiki_dir.join(format!("{}.md", new_sanitized));

    if !old_path.exists() {
        return Err(AdkError::tool(format!(
            "Wiki page '{}' not found.",
            args.old_title
        )));
    }

    if new_path.exists() {
        return Err(AdkError::tool(format!(
            "Destination page '{}' already exists.",
            args.new_title
        )));
    }

    // Create parent directories for new path if needed
    if let Some(parent) = new_path.parent()
        && !parent.exists() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AdkError::tool(format!("Failed to create parent directories: {}", e))
            })?;
        }

    // Rename the file
    fs::rename(&old_path, &new_path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to rename file: {}", e)))?;

    // Update links in all other files
    let mut links_updated = 0;
    let old_link_exact = format!("[[{}]]", args.old_title);
    // Also try to match the sanitized version or just the filename if it was moved
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

    Ok(json!({
        "status": "success",
        "message": format!("Renamed '{}' to '{}'.", args.old_title, args.new_title),
        "files_updated_with_new_links": links_updated
    }))
}

#[tool]
async fn sanitize_wiki_vault(_args: SanitizeWikiVaultArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut files_to_process = Vec::new();

    // Pass 1: Identify files that need renaming
    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&wiki_dir, path);
            files_to_process.push(path.to_path_buf());

            // Only process the actual filename, keep the parent path
            let file_stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let new_stem = to_title_case(&file_stem);

            if file_stem != new_stem {
                let mut new_title = relative_title.clone();
                // Replace just the filename part in the relative path
                if let Some(pos) = new_title.rfind(&file_stem) {
                    new_title.replace_range(pos..pos + file_stem.len(), &new_stem);
                }
                rename_map.insert(relative_title, new_title);
            }
        }
    }

    let mut renamed_count = 0;
    let mut links_updated_count = 0;

    // Pass 2: Rename files
    for (old_title, new_title) in &rename_map {
        let old_path = wiki_dir.join(format!("{}.md", old_title));
        let new_path = wiki_dir.join(format!("{}.md", new_title));

        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path).await.map_err(|e| {
                AdkError::tool(format!(
                    "Failed to rename {:?} to {:?}: {}",
                    old_path, new_path, e
                ))
            })?;
            renamed_count += 1;
        }
    }

    // Pass 3: Update contents (wikilinks) in ALL files
    // Use the updated file paths if they were renamed
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
                // Match exact wikilinks e.g., [[old-title]] -> [[New Title]]
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
            }
        }
    }

    Ok(json!({
        "status": "success",
        "message": "Vault cleanup complete.",
        "files_renamed": renamed_count,
        "links_updated": links_updated_count,
        "renamed_mapping": rename_map
    }))
}

pub fn wiki_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AddWikiPage),
        Arc::new(GetWikiPage),
        Arc::new(ListWikiPages),
        Arc::new(SearchWiki),
        Arc::new(SummarizeWiki),
        Arc::new(SearchWikiByTag),
        Arc::new(GetWikiGraph),
        Arc::new(CreateDailyNote),
        Arc::new(SanitizeWikiVault),
        Arc::new(GetBacklinks),
        Arc::new(CheckBrokenLinks),
        Arc::new(RenameWikiPage),
        Arc::new(ApplyTemplate),
    ]
}
