use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use tokio::fs;
use walkdir::WalkDir;
use crate::utils::{get_wiki_dir, sandbox, ignore::NamiIgnore};

/// Returns the Axum Router for the API.
use crate::modes::command_registry::CommandRegistry;
// ... (rest of imports)

pub fn api_router() -> Router {
    Router::new()
        .route("/api/workspace/files", get(list_folder_contents))
        .route("/api/workspace/files/{*path}", get(list_folder_contents))
        .route("/api/workspace/read/{*path}", get(read_workspace_file))
        .route("/api/wiki/pages", get(list_wiki_pages))
        .route("/api/wiki/pages/{*title}", get(read_wiki_page))
        .route("/api/commands", get(get_commands))
}

async fn get_commands() -> impl IntoResponse {
    match CommandRegistry::load_from_config("config.toml") {
        Ok(registry) => Json(json!(registry.commands)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load commands").into_response(),
    }
}


const IGNORED_DIRS: &[&str] = &[
    ".venv", ".cache", ".config", ".local", ".npm", ".rustup", ".git"
];

/// Lists contents of a workspace directory.
/// 
/// If `path` is provided, lists contents of the specified relative path.
/// If `path` is not provided, lists the workspace root contents.
async fn list_folder_contents(path: Option<Path<String>>) -> impl IntoResponse {
    let path = path.map(|p| p.0).unwrap_or_default();
    let folder_path = match sandbox(&path).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::FORBIDDEN, e.to_string()).into_response(),
    };

    if !folder_path.is_dir() {
        return (StatusCode::BAD_REQUEST, "Path is not a directory").into_response();
    }

    let ignore = NamiIgnore::load().await;
    let mut entries = Vec::new();

    let mut read_dir = match tokio::fs::read_dir(&folder_path).await {
        Ok(rd) => rd,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        let relative = match path.strip_prefix(&folder_path) {
            Ok(r) => r.to_owned(),
            Err(_) => continue,
        };

        // Skip hardcoded ignored dirs
        if let Some(name) = relative.file_name() {
            if IGNORED_DIRS.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
        }

        if ignore.is_ignored(&relative) {
            continue;
        }

        let name = relative.to_string_lossy().replace("\\", "/");
        let is_dir = path.is_dir();

        entries.push(json!({
            "name": if path.is_dir() { format!("{}/", name) } else { name },
            "type": if is_dir { "folder" } else { "file" }
        }));
    }

    Json(json!({ "entries": entries })).into_response()
}

/// Reads the content of a file within the workspace.
async fn read_workspace_file(Path(path): Path<String>) -> impl IntoResponse {
    let full_path = match sandbox(&path).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::FORBIDDEN, e.to_string()).into_response(),
    };

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    match fs::read_to_string(&full_path).await {
        Ok(content) => Json(json!({ "content": content })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Read failed: {}", e)).into_response(),
    }
}

/// Lists all Markdown pages available in the wiki.
async fn list_wiki_pages() -> impl IntoResponse {
    let wiki_dir = match get_wiki_dir().await {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut pages = Vec::new();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative = path.strip_prefix(&wiki_dir).unwrap_or(path);
            pages.push(relative.with_extension("").to_string_lossy().replace("\\", "/"));
        }
    }

    Json(json!({ "pages": pages })).into_response()
}

/// Reads the content of a specific wiki page by title.
async fn read_wiki_page(Path(title): Path<String>) -> impl IntoResponse {
    match get_wiki_dir().await {
        Ok(_) => (),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Use sandbox logic to ensure title doesn't escape wiki directory
    let filename = format!("{}.md", title);
    let full_path = match sandbox(&format!("wiki/{}", filename)).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::FORBIDDEN, e.to_string()).into_response(),
    };

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "Wiki page not found").into_response();
    }

    match fs::read_to_string(&full_path).await {
        Ok(content) => Json(json!({ "title": title, "content": content })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Read failed: {}", e)).into_response(),
    }
}
