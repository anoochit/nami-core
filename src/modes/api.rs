use axum::{
    extract::{Path, Multipart, Request},
    http::{StatusCode, header::HeaderValue},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use serde_json::json;
use tokio::fs;
use walkdir::WalkDir;
use crate::utils::{get_km_dir, get_workspace_dir, sandbox, ignore::NamiIgnore, get_nami_dir};

#[tracing::instrument]
async fn list_sessions() -> impl IntoResponse {
    use sqlx::Row;

    let pool = crate::utils::db_pool();

    let sessions = match sqlx::query("SELECT session_id, app_name, user_id, created_at FROM sessions WHERE EXISTS (SELECT 1 FROM events WHERE events.session_id = sessions.session_id) ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let mut results = Vec::new();
            for row in rows {
                results.push(json!({
                    "session_id": row.get::<String, _>("session_id"),
                    "app_name": row.get::<String, _>("app_name"),
                    "user_id": row.get::<String, _>("user_id"),
                    "created_at": row.get::<String, _>("created_at"),
                }));
            }
            results
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to query sessions: {}", e)).into_response(),
    };

    Json(json!({ "sessions": sessions })).into_response()
}

/// Returns the Axum Router for the API.
use crate::modes::command_registry::CommandRegistry;

#[tracing::instrument]
async fn get_session_messages(Path(session_id): Path<String>) -> impl IntoResponse {
    use sqlx::Row;

    let pool = crate::utils::db_pool();

    let messages = match sqlx::query("SELECT llm_response, author, timestamp FROM events WHERE session_id = ? ORDER BY timestamp ASC")
        .bind(session_id)
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let mut results = Vec::new();
            for row in rows {
                results.push(json!({
                    "llm_response": row.get::<String, _>("llm_response"),
                    "author": row.get::<String, _>("author"),
                    "timestamp": row.get::<String, _>("timestamp"),
                }));
            }
            results
        },
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to query messages: {}", e)).into_response(),
    };

    Json(json!({ "messages": messages })).into_response()
}

#[tracing::instrument]
async fn create_session_handler(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let app_name = payload["appName"].as_str().unwrap_or("nami");
    let user_id = payload["userId"].as_str().unwrap_or("user1");
    let session_id = payload["sessionId"].as_str().unwrap_or("");
    
    if session_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing sessionId").into_response();
    }

    let pool = crate::utils::db_pool();

    let now = chrono::Utc::now().to_rfc3339();

    match sqlx::query("INSERT INTO sessions (app_name, user_id, session_id, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(app_name)
        .bind(user_id)
        .bind(session_id)
        .bind("{}") // Empty state
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
    {
        Ok(_) => Json(json!({ "session_id": session_id })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create session: {}", e)).into_response(),
    }
}

pub fn api_router() -> Router {
    Router::new()
        .route("/api/workspace/files", get(list_root_folder_contents))
        .route("/api/workspace/files/{*path}", get(list_sub_folder_contents))
        .route("/api/workspace/read/{*path}", get(read_workspace_file))
        .route("/api/workspace/read-binary/{*path}", get(read_workspace_binary))
        .route("/api/workspace/upload", post(upload_file))
        .route("/api/km/pages", get(list_km_pages))
        .route("/api/km/pages/{*title}", get(read_km_page))
        .route("/api/commands", get(get_commands))
        .route("/api/sessions/create", post(create_session_handler))
        .route("/api/sessions/list", get(list_sessions))
        .route("/api/sessions/{session_id}/messages", get(get_session_messages))
        .route("/api/scheduler", get(list_scheduler_tasks))
        .route("/api/scheduler/add", post(add_scheduler_task))
        .route("/api/scheduler/{id}", delete(delete_scheduler_task))
        .route("/api/scheduler/{id}/toggle", post(toggle_scheduler_task))
        .route("/api/todos", get(list_todos_handler).post(add_todo_handler))
        .route("/api/todos/{id}/toggle", post(toggle_todo_handler))
        .route("/api/todos/{id}", delete(delete_todo_handler))
        .route("/api/workspaces", get(get_workspaces))
        .route("/api/workspaces/select", post(select_workspace))
        .route("/api/workspaces/add", post(add_workspace))
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


#[tracing::instrument]
async fn get_commands() -> impl IntoResponse {
    let config_path = get_nami_dir().join("config.toml");
    match CommandRegistry::load_from_config(&config_path.to_string_lossy()) {
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

/// Lists all Markdown pages available in the knowledge vault.
async fn list_km_pages() -> impl IntoResponse {
    let km_dir = match get_km_dir().await {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut pages = Vec::new();

    for entry in WalkDir::new(&km_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative = path.strip_prefix(&km_dir).unwrap_or(path);
            pages.push(relative.with_extension("").to_string_lossy().replace("\\", "/"));
        }
    }

    Json(json!({ "pages": pages })).into_response()
}

/// Reads the content of a specific knowledge page by title.
async fn read_km_page(Path(title): Path<String>) -> impl IntoResponse {
    let km_dir = match get_km_dir().await {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Ensure title does not contain parent directory escape sequences
    let clean_title = title.trim_start_matches(['/', '\\']);
    let mut joined = km_dir.clone();
    joined.push(format!("{}.md", clean_title));

    // Normalize path to resolve any parent/current directory segments
    let mut normalized = std::path::PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            c => normalized.push(c),
        }
    }

    // Safety guard: Must start with the global knowledge directory root
    if !normalized.starts_with(&km_dir) {
        return (StatusCode::FORBIDDEN, "Security Error: Attempt to escape knowledge vault sandbox").into_response();
    }

    if !normalized.exists() {
        return (StatusCode::NOT_FOUND, "Knowledge page not found").into_response();
    }

    match fs::read_to_string(&normalized).await {
        Ok(content) => Json(json!({ "title": title, "content": content })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Read failed: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn get_workspaces() -> impl IntoResponse {
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let active = crate::utils::clean_unc_path(std::fs::canonicalize(&current_dir).unwrap_or(current_dir))
        .to_string_lossy()
        .replace('\\', "/");
    let list = vec![active.clone()];

    Json(json!({
        "active": active,
        "list": list
    })).into_response()
}

#[derive(serde::Deserialize, Debug)]
struct WorkspacePathPayload {
    path: String,
}

#[tracing::instrument]
async fn add_workspace(Json(payload): Json<WorkspacePathPayload>) -> impl IntoResponse {
    // No-op success for backward compatibility with frontend WebUI
    Json(json!({ "status": "success", "added": payload.path })).into_response()
}

#[derive(serde::Deserialize, Debug)]
struct WorkspaceSelectPayload {
    index_or_path: String,
}

#[tracing::instrument]
async fn select_workspace(Json(payload): Json<WorkspaceSelectPayload>) -> impl IntoResponse {
    // No-op success for backward compatibility with frontend WebUI
    Json(json!({ "status": "success", "active": payload.index_or_path })).into_response()
}

use crate::tools::scheduler::{load_schedule, save_schedule, ScheduledTask};
use cron::Schedule;
use std::str::FromStr;
use uuid::Uuid;

#[tracing::instrument]
async fn list_scheduler_tasks() -> impl IntoResponse {
    match load_schedule().await {
        Ok(tasks) => Json(json!({ "tasks": tasks })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load schedule: {}", e)).into_response(),
    }
}

#[derive(serde::Deserialize, Debug)]
struct AddTaskPayload {
    goal: String,
    cron_expr: String,
}

#[tracing::instrument(skip(payload))]
async fn add_scheduler_task(Json(payload): Json<AddTaskPayload>) -> impl IntoResponse {
    // Validate cron expression
    if let Err(e) = Schedule::from_str(&payload.cron_expr) {
        return (StatusCode::BAD_REQUEST, format!("Invalid cron expression: {}", e)).into_response();
    }

    let mut tasks = match load_schedule().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load schedule: {}", e)).into_response(),
    };

    let id = Uuid::new_v4().to_string();
    let new_task = ScheduledTask {
        id: id.clone(),
        goal: payload.goal,
        cron_expr: payload.cron_expr,
        last_run: None,
        is_active: true,
    };

    tasks.push(new_task);

    match save_schedule(&tasks).await {
        Ok(_) => Json(json!({ "status": "success", "id": id })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save schedule: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn delete_scheduler_task(Path(id): Path<String>) -> impl IntoResponse {
    let mut tasks = match load_schedule().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load schedule: {}", e)).into_response(),
    };

    let original_len = tasks.len();
    tasks.retain(|t| t.id != id);

    if tasks.len() == original_len {
        return (StatusCode::NOT_FOUND, "Task not found").into_response();
    }

    match save_schedule(&tasks).await {
        Ok(_) => Json(json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save schedule: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn toggle_scheduler_task(Path(id): Path<String>) -> impl IntoResponse {
    let mut tasks = match load_schedule().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load schedule: {}", e)).into_response(),
    };

    let mut found = false;
    for task in tasks.iter_mut() {
        if task.id == id {
            task.is_active = !task.is_active;
            found = true;
            break;
        }
    }

    if !found {
        return (StatusCode::NOT_FOUND, "Task not found").into_response();
    }

    match save_schedule(&tasks).await {
        Ok(_) => Json(json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save schedule: {}", e)).into_response(),
    }
}

#[derive(serde::Deserialize, Debug)]
struct AddTodoPayload {
    description: String,
}

#[tracing::instrument]
async fn list_todos_handler() -> impl IntoResponse {
    match crate::tools::todo::load_todos().await {
        Ok(todos) => Json(json!({ "todos": todos })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load todos: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn add_todo_handler(Json(payload): Json<AddTodoPayload>) -> impl IntoResponse {
    let mut todos = match crate::tools::todo::load_todos().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load todos: {}", e)).into_response(),
    };

    let next_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let new_todo = crate::tools::todo::Todo {
        id: next_id,
        description: payload.description.clone(),
        done: false,
    };
    todos.push(new_todo.clone());

    match crate::tools::todo::save_todos(&todos).await {
        Ok(_) => Json(json!({ "status": "success", "todo": new_todo })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save todos: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn toggle_todo_handler(Path(id): Path<usize>) -> impl IntoResponse {
    let mut todos = match crate::tools::todo::load_todos().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load todos: {}", e)).into_response(),
    };

    let mut found = false;
    for todo in todos.iter_mut() {
        if todo.id == id {
            todo.done = !todo.done;
            found = true;
            break;
        }
    }

    if !found {
        return (StatusCode::NOT_FOUND, "Todo item not found").into_response();
    }

    match crate::tools::todo::save_todos(&todos).await {
        Ok(_) => Json(json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save todos: {}", e)).into_response(),
    }
}

#[tracing::instrument]
async fn delete_todo_handler(Path(id): Path<usize>) -> impl IntoResponse {
    let mut todos = match crate::tools::todo::load_todos().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load todos: {}", e)).into_response(),
    };

    let original_len = todos.len();
    todos.retain(|t| t.id != id);

    if todos.len() == original_len {
        return (StatusCode::NOT_FOUND, "Todo item not found").into_response();
    }

    match crate::tools::todo::save_todos(&todos).await {
        Ok(_) => Json(json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save todos: {}", e)).into_response(),
    }
}
