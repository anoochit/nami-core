use adk_rust::Tool;
use adk_rust::serde::{Deserialize, Serialize};
use adk_tool::AdkError;
use chrono::{Datelike, Timelike, Utc};
use regex::Regex;
use schemars::JsonSchema;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use walkdir::WalkDir;

mod read;
mod write;
mod search;
mod list;
mod misc;

#[derive(Deserialize, JsonSchema)]
struct WikiPageArgs {
    /// The title of the wiki page/concept (e.g., 'project-notes'). This will be used as the filename.
    title: String,
    /// Optional: The starting line number to read (1-indexed, inclusive).
    start_line: Option<usize>,
    /// Optional: The ending line number to read (1-indexed, inclusive).
    end_line: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
struct AddWikiArgs {
    /// The title of the wiki page/concept.
    title: String,
    /// The content in Markdown format (including OKF v0.2 frontmatter if applicable).
    content: String,
    /// Optional: Concept type according to OKF v0.2 specification (e.g., 'Concept', 'Playbook', 'Metric', 'Attested Computation'). Defaults to 'Concept'.
    r#type: Option<String>,
    /// Optional: Brief description of the concept for OKF v0.2 metadata.
    description: Option<String>,
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
    /// Optional: Filter results by OKF concept type (e.g., 'Playbook', 'Metric', 'Concept').
    r#type: Option<String>,
    /// Optional: Filter results by OKF status ('draft', 'stable', 'deprecated').
    status: Option<String>,
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
struct ListWikiPagesArgs {
    /// Optional: Filter listed pages by OKF concept type (e.g., 'Concept', 'Metric', 'Playbook', 'Attested Computation').
    r#type: Option<String>,
    /// Optional: Filter listed pages by OKF status ('draft', 'stable', 'deprecated').
    status: Option<String>,
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

#[derive(Deserialize, JsonSchema, Debug)]
struct SummarizeWikiArgs {}

// --- OKF v0.2 DATA STRUCTURES ---

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkfGenerated {
    pub by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkfVerifiedItem {
    pub by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OkfVerified {
    Single(OkfVerifiedItem),
    List(Vec<OkfVerifiedItem>),
}

impl OkfVerified {
    pub fn as_list(&self) -> Vec<OkfVerifiedItem> {
        match self {
            OkfVerified::Single(item) => vec![item.clone()],
            OkfVerified::List(list) => list.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkfSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub resource: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OkfUsageWindow {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OkfFrontmatter {
    /// REQUIRED by OKF v0.2 (§4.1). Defaults to "Concept" for untagged legacy docs.
    #[serde(default = "default_concept_type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<OkfSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_window: Option<OkfUsageWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<OkfGenerated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<OkfVerified>,
    /// status: draft | stable | deprecated (default: stable)
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_after: Option<String>,
    // Attested Computation fields (§10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attester: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl Default for OkfFrontmatter {
    fn default() -> Self {
        Self {
            r#type: default_concept_type(),
            title: None,
            description: None,
            resource: None,
            tags: Vec::new(),
            sources: None,
            usage_window: None,
            generated: None,
            verified: None,
            status: default_status(),
            stale_after: None,
            runtime: None,
            parameters: None,
            computation: None,
            executor: None,
            attester: None,
            extra: HashMap::new(),
        }
    }
}

fn default_concept_type() -> String {
    "Concept".to_string()
}

fn default_status() -> String {
    "stable".to_string()
}

// --- CACHING STRUCTURES ---

#[derive(Clone, Debug)]
pub struct WikiPageMetadata {
    pub title: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub links: Vec<String>, // Links parsed from content: both standard [Label](/concept.md) and [[wikilinks]]
    pub frontmatter: HashMap<String, String>,
    pub okf: OkfFrontmatter,
    pub trust_tier: String, // "unverified", "machine-confirmed", "human-reviewed"
    pub is_stale: bool,
    pub content: Option<String>,
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
    let mut okf = OkfFrontmatter::default();

    let mut main_content = &content[..];
    if let Some(rest) = content.strip_prefix("---")
        && let Some(end_idx) = rest.find("---")
    {
        let fm_str = &rest[..end_idx];
        main_content = &rest[end_idx + 3..];

        // Attempt full OKF YAML frontmatter deserialization
        if let Ok(parsed_okf) = serde_yaml::from_str::<OkfFrontmatter>(fm_str) {
            okf = parsed_okf;
        }

            // Also populate legacy frontmatter map for backward compatibility
            for line in fm_str.lines() {
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

    // Merge OKF tags into tag list
    for okf_tag in &okf.tags {
        let t = okf_tag.to_lowercase();
        if !tags.contains(&t) {
            tags.push(t);
        }
    }

    // Parse Markdown standard links: [Label](/path.md) or [Label](path.md)
    let md_link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    for cap in md_link_re.captures_iter(main_content) {
        let link_path = cap[2].trim();
        // Ignore external HTTP/HTTPS links
        if !link_path.starts_with("http://") && !link_path.starts_with("https://") && !link_path.starts_with('#') {
            let clean_link = link_path.trim_start_matches('/').trim_end_matches(".md");
            let target = sanitize_title(clean_link);
            if !target.is_empty() && !links.contains(&target) {
                links.push(target);
            }
        }
    }

    // Parse wikilinks: [[PageTitle]] or [[PageTitle|Alias]]
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

    // Derive trust tier according to OKF §5.3
    let trust_tier = match &okf.verified {
        None => "unverified".to_string(),
        Some(v) => {
            let list = v.as_list();
            if list.is_empty() {
                "unverified".to_string()
            } else if list.iter().any(|item| item.by.starts_with("human:")) {
                "human-reviewed".to_string()
            } else {
                "machine-confirmed".to_string()
            }
        }
    };

    // Calculate staleness according to OKF §5.5
    let is_stale = if let Some(ref stale_date) = okf.stale_after {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        today >= *stale_date
    } else {
        false
    };

    Ok(WikiPageMetadata {
        title,
        path: path.to_path_buf(),
        tags,
        links,
        frontmatter,
        okf,
        trust_tier,
        is_stale,
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

pub fn wiki_tools() -> Vec<Arc<dyn Tool>> {
    let mut all = Vec::new();
    all.extend(read::tools());
    all.extend(write::tools());
    all.extend(search::tools());
    all.extend(list::tools());
    all.extend(misc::tools());
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nami-okf-test-{}", label));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_okf_v02_frontmatter_and_derives_metadata() {
        let wiki_dir = create_test_dir("okf-parse");
        let concept_file = wiki_dir.join("revenue.md");

        let okf_md = r#"---
type: Attested Computation
title: Revenue for fiscal year
description: Recognized revenue for a fiscal year.
status: stable
runtime: bigquery
stale_after: 2099-12-31
generated:
  by: reference_agent/gemini-2.5-pro
  at: 2026-06-20T22:53:05Z
verified:
  - by: human:ahormati
    at: 2026-06-25T09:00:00Z
sources:
  - id: rev-policy
    resource: https://wiki.acme/finance/revenue-recognition
    title: Revenue recognition policy
tags: [finance, revenue]
---

# Computation

    SELECT SUM(amount) AS revenue
    FROM finance.recognized_revenue

See also [Customers Table](/tables/customers.md) or [[Orders Page]].
"#;

        std::fs::write(&concept_file, okf_md).unwrap();

        let metadata = parse_wiki_file_sync(&wiki_dir, &concept_file).unwrap();

        assert_eq!(metadata.okf.r#type, "Attested Computation");
        assert_eq!(metadata.okf.title.as_deref(), Some("Revenue for fiscal year"));
        assert_eq!(metadata.okf.description.as_deref(), Some("Recognized revenue for a fiscal year."));
        assert_eq!(metadata.okf.status, "stable");
        assert_eq!(metadata.okf.runtime.as_deref(), Some("bigquery"));
        assert_eq!(metadata.trust_tier, "human-reviewed");
        assert_eq!(metadata.is_stale, false);
        assert!(metadata.tags.contains(&"finance".to_string()));
        assert!(metadata.tags.contains(&"revenue".to_string()));
        assert!(metadata.links.iter().any(|l| l.to_lowercase().contains("customers")));
        assert!(metadata.links.iter().any(|l| l.to_lowercase().contains("orders")));

        let _ = std::fs::remove_dir_all(&wiki_dir);
    }

    #[test]
    fn parses_unverified_untyped_concept_fallback() {
        let wiki_dir = create_test_dir("okf-fallback");
        let concept_file = wiki_dir.join("note.md");

        let legacy_md = r#"---
title: Quick Note
tags: [ideas]
---

Just a simple note.
"#;

        std::fs::write(&concept_file, legacy_md).unwrap();

        let metadata = parse_wiki_file_sync(&wiki_dir, &concept_file).unwrap();

        assert_eq!(metadata.okf.r#type, "Concept");
        assert_eq!(metadata.okf.status, "stable");
        assert_eq!(metadata.trust_tier, "unverified");
        assert_eq!(metadata.is_stale, false);

        let _ = std::fs::remove_dir_all(&wiki_dir);
    }
}
