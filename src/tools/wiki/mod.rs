use crate::utils::get_wiki_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use chrono::{Datelike, Timelike, Utc};
use regex::Regex;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use tokio::fs;
use walkdir::WalkDir;

#[derive(Deserialize, JsonSchema)]
struct WikiPageArgs {
    /// The title of the wiki page (e.g., 'project-notes'). This will be used as the filename.
    title: String,
    /// Optional: The starting line number to read (1-indexed, inclusive).
    start_line: Option<usize>,
    /// Optional: The ending line number to read (1-indexed, inclusive).
    end_line: Option<usize>,
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
    /// Optional: Maximum number of search results to return (defaults to 50).
    limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchWikiByTagArgs {
    /// The tag to search for (e.g., 'rust', 'project-ideas'). Do not include the '#' symbol.
    tag: String,
}

#[derive(Deserialize, JsonSchema)]
struct GlobFindWikiArgs {
    /// The glob pattern to match against wiki page paths or titles (e.g. "Projects/*.md" or "**/ideas.md").
    pattern: String,
}

#[derive(Deserialize, JsonSchema)]
struct ListWikiPagesArgs {}

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

#[derive(Deserialize, JsonSchema, Debug)]
struct SummarizeWikiArgs {}

// --- CACHING STRUCTURES ---

#[derive(Clone, Debug)]
struct WikiPageMetadata {
    #[allow(dead_code)]
    title: String,
    path: PathBuf,
    tags: Vec<String>,
    links: Vec<String>, // Wikilinks parsed from the content: [[PageTitle]]
    frontmatter: HashMap<String, String>,
    content: Option<String>,
}

struct WikiCache {
    pages: HashMap<String, WikiPageMetadata>,
    initialized: bool,
}

static WIKI_CACHE: OnceLock<RwLock<WikiCache>> = OnceLock::new();

fn get_cache() -> &'static RwLock<WikiCache> {
    WIKI_CACHE.get_or_init(|| {
        RwLock::new(WikiCache {
            pages: HashMap::new(),
            initialized: false,
        })
    })
}

async fn ensure_cache_initialized(wiki_dir: &Path) -> std::result::Result<(), AdkError> {
    let is_initialized = {
        let cache = get_cache().read().map_err(|e| AdkError::tool(format!("Failed to acquire cache read lock: {}", e)))?;
        cache.initialized
    };

    if is_initialized {
        return Ok(());
    }

    let mut cache = get_cache().write().map_err(|e| AdkError::tool(format!("Failed to acquire cache write lock: {}", e)))?;
    cache.pages.clear();
    for entry in WalkDir::new(wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(wiki_dir, path);
            // Parse synchronously to avoid holding lock across await boundaries
            if let Ok(metadata) = parse_wiki_file_sync(wiki_dir, path) {
                cache.pages.insert(relative_title, metadata);
            }
        }
    }
    cache.initialized = true;
    Ok(())
}

fn parse_wiki_file_sync(wiki_dir: &Path, path: &Path) -> anyhow::Result<WikiPageMetadata> {
    let content = std::fs::read_to_string(path)?;
    let title = get_relative_title(wiki_dir, path);
    
    let mut tags = Vec::new();
    let mut links = Vec::new();
    let mut frontmatter = HashMap::new();

    let mut main_content = &content[..];
    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            let fm_content = &content[3..end_idx + 3];
            main_content = &content[end_idx + 6..];
            for line in fm_content.lines() {
                if let Some((key, val)) = line.split_once(':') {
                    let k = key.trim().to_lowercase();
                    let v = val.trim().trim_matches('"').trim_matches('\'').to_string();
                    if k == "tags" {
                        if v.starts_with('[') && v.ends_with(']') {
                            for tag in v[1..v.len()-1].split(',') {
                                let t = tag.trim().to_lowercase();
                                if !t.is_empty() {
                                    tags.push(t);
                                }
                            }
                        } else {
                            for tag in v.split(',') {
                                let t = tag.trim().to_lowercase();
                                if !t.is_empty() {
                                    tags.push(t);
                                }
                            }
                        }
                    } else {
                        frontmatter.insert(k, v);
                    }
                } else if line.trim().starts_with("- ") {
                    let val = line.trim()[2..].trim().trim_matches('"').trim_matches('\'').to_lowercase();
                    if !val.is_empty() {
                        tags.push(val);
                    }
                }
            }
        }
    }

    // Parse wikilinks
    let link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    for cap in link_re.captures_iter(main_content) {
        let link_target = sanitize_title(&cap[1]);
        if !links.contains(&link_target) {
            links.push(link_target);
        }
    }

    // Parse inline #tags
    let tag_re = Regex::new(r"#([a-zA-Z0-9_\-]+)").unwrap();
    for cap in tag_re.captures_iter(main_content) {
        let tag = cap[1].to_lowercase();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    Ok(WikiPageMetadata {
        title,
        path: path.to_path_buf(),
        tags,
        links,
        frontmatter,
        content: Some(content),
    })
}

// --- GIT VERSIONING HELPER ---

fn git_auto_commit(wiki_dir: &Path, file_path: &Path, action_message: &str) {
    let git_dir = wiki_dir.join(".git");
    let is_git = git_dir.exists() || {
        let mut p = wiki_dir.to_path_buf();
        let mut found = false;
        while p.pop() {
            if p.join(".git").exists() {
                found = true;
                break;
            }
        }
        found
    };

    if is_git {
        let _ = std::process::Command::new("git")
            .arg("add")
            .arg(file_path)
            .current_dir(wiki_dir)
            .output();

        let _ = std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(action_message)
            .current_dir(wiki_dir)
            .output();
    }
}

// --- TEMPLATE HELPER ---

fn expand_template_variables(template: &str, title: &str) -> String {
    let today = Utc::now();
    let date_str = format!("{}-{:02}-{:02}", today.year(), today.month(), today.day());
    let time_str = format!("{:02}:{:02}", today.hour(), today.minute());
    let week_str = format!("{}", today.iso_week().week());
    
    template
        .replace("{{date}}", &date_str)
        .replace("{{time}}", &time_str)
        .replace("{{title}}", title)
        .replace("{{week}}", &week_str)
}

// --- TITLE CASE HELPERS ---

fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    let mut last_was_space = false;
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
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

fn sanitize_title(title: &str) -> String {
    let sanitized_path = title.trim().replace("\\", "/");
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

fn get_relative_title(wiki_dir: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(wiki_dir)
        .unwrap_or(file_path)
        .with_extension("")
        .to_string_lossy()
        .replace("\\", "/")
}

// --- TOOLS ---

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

/// Retrieves the content of a specific wiki page. Supporting Token Safety & Line Pagination.
#[tool]
async fn get_wiki_page(args: WikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = sanitize_title(&args.title);
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

/// Searches for a keyword across all wiki pages recursively with caps.
#[tool]
async fn search_wiki(args: SearchWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let query_lower = args.query.to_lowercase();
    let limit = args.limit.unwrap_or(50);

    let regex_pattern = if args.use_regex.unwrap_or(false) {
        Regex::new(&args.query).ok()
    } else {
        None
    };

    let headers_only = args.headers_only.unwrap_or(false);

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        if matches.len() >= limit {
            break;
        }

        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
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
                let relative_path = path.strip_prefix(&wiki_dir).unwrap_or(path).to_string_lossy().replace("\\", "/");
                matches.push(json!({
                    "title": get_relative_title(&wiki_dir, path),
                    "path": format!("wiki/{}", relative_path)
                }));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": "No matches found in wiki." }))
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
        Arc::new(DeleteWikiPage),
        Arc::new(GlobFindWiki),
        Arc::new(ApplyTemplate),
    ]
}
