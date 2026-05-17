use axum::{
    extract::{Path, Multipart, Request},
    http::{StatusCode, header::HeaderValue},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tokio::fs;
use walkdir::WalkDir;
use crate::utils::{get_wiki_dir, get_workspace_dir, sandbox, ignore::NamiIgnore};
use sqlx::{SqlitePool, Row};

/// Returns the Axum Router for the API.
use crate::modes::command_registry::CommandRegistry;

pub fn api_router() -> Router {
    Router::new()
        .route("/api/workspace/files", get(list_root_folder_contents))
        .route("/api/workspace/files/{*path}", get(list_sub_folder_contents))
        .route("/api/workspace/read/{*path}", get(read_workspace_file))
        .route("/api/workspace/read-binary/{*path}", get(read_workspace_binary))
        .route("/api/workspace/upload", post(upload_file))
        .route("/api/wiki/pages", get(list_wiki_pages))
        .route("/api/wiki/pages/{*title}", get(read_wiki_page))
        .route("/api/commands", get(get_commands))
        .route("/api/sessions", get(list_sessions))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(secure_headers))
}

async fn secure_headers(req: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(req).await.into_response();
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
    response
}

async fn auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    if let Ok(expected_key) = std::env::var("NAMI_API_KEY") {
        let provided_key = req.headers()
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok());

        if let Some(key) = provided_key {
            if key == expected_key {
                return next.run(req).await.into_response();
            }
        }
        
        (StatusCode::UNAUTHORIZED, "Invalid or missing X-API-Key").into_response()
    } else {
        // If NAMI_API_KEY is not set, allow all requests
        next.run(req).await.into_response()
    }
}

async fn list_sessions() -> impl IntoResponse {
    let db_path = "sessions.db";
    let pool = match SqlitePool::connect(&format!("sqlite:{}?mode=ro", db_path)).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Connect Error: {}", e)).into_response(),
    };

    let result = sqlx::query("SELECT session_id, app_name, user_id, created_at, updated_at FROM sessions")
        .fetch_all(&pool)
        .await;

    match result {
        Ok(rows) => {
            let sessions: Vec<_> = rows
                .iter()
                .filter(|row| row.get::<String, _>("session_id") != "background_tasks")
                .map(|row| {
                    json!({
                        "session_id": row.get::<String, _>("session_id"),
                        "app_name": row.get::<String, _>("app_name"),
                        "user_id": row.get::<String, _>("user_id"),
                        "created_at": row.get::<String, _>("created_at"),
                        "updated_at": row.get::<String, _>("updated_at"),
                    })
                })
                .collect();
            Json(sessions).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error: {}", e)).into_response(),
    }
}

async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    let mut uploaded_paths = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or_default().to_string();
        if file_name.is_empty() {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => return (StatusCode::BAD_REQUEST, format!("Failed to read field: {}", e)).into_response(),
        };

        // Ensure uploads directory exists
        let root = match get_workspace_dir().await {
            Ok(r) => r,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        let upload_dir = root.join("uploads");
        if !upload_dir.exists() {
            if let Err(e) = fs::create_dir_all(&upload_dir).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create uploads dir: {}", e)).into_response();
            }
        }

        // Clean filename to prevent path traversal
        let safe_name = std::path::Path::new(&file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        
        let file_path = upload_dir.join(safe_name);

        if let Err(e) = fs::write(&file_path, data).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e)).into_response();
        }

        uploaded_paths.push(format!("uploads/{}", safe_name));
    }

    Json(json!({ "paths": uploaded_paths })).into_response()
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

/// Lists contents of a workspace root.
async fn list_root_folder_contents() -> impl IntoResponse {
    list_folder_contents_internal("".to_string()).await
}

/// Lists contents of a workspace directory.
async fn list_sub_folder_contents(Path(path): Path<String>) -> impl IntoResponse {
    list_folder_contents_internal(path).await
}

async fn list_folder_contents_internal(path: String) -> impl IntoResponse {
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

    entries.sort_by(|a, b| {
        let type_a = a["type"].as_str().unwrap();
        let type_b = b["type"].as_str().unwrap();
        if type_a != type_b {
            type_b.cmp(type_a) // folder (1) comes before file (0)
        } else {
            a["name"].as_str().unwrap().cmp(b["name"].as_str().unwrap())
        }
    });

    Json(json!({ "entries": entries })).into_response()
}

/// Reads the content of a file as binary data.
async fn read_workspace_binary(Path(path): Path<String>) -> impl IntoResponse {
    let full_path = match sandbox(&path).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::FORBIDDEN, e.to_string()).into_response(),
    };

    if !full_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let content = match fs::read(&full_path).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Read failed: {}", e)).into_response(),
    };

    let content_type = mime_guess::from_path(&full_path).first_or_octet_stream();

    (
        [(axum::http::header::CONTENT_TYPE, content_type.as_ref())],
        content,
    ).into_response()
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
