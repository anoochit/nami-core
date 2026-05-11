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
use crate::utils::{get_workspace_dir, get_wiki_dir, sandbox, ignore::NamiIgnore};

pub fn api_router() -> Router {
    Router::new()
        .route("/api/workspace/files", get(list_workspace_files))
        .route("/api/workspace/files/*path", get(read_workspace_file))
        .route("/api/wiki/pages", get(list_wiki_pages))
        .route("/api/wiki/pages/*title", get(read_wiki_page))
}

async fn list_workspace_files() -> impl IntoResponse {
    let root = match get_workspace_dir().await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let ignore = NamiIgnore::load().await;
    let mut files = Vec::new();

    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative = path.strip_prefix(&root).unwrap_or(path);
            if !ignore.is_ignored(relative) {
                files.push(relative.to_string_lossy().replace("\\", "/"));
            }
        }
    }

    Json(json!({ "files": files })).into_response()
}

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
