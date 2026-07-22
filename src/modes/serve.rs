use crate::agent::get_compaction_config;
use crate::modes::command_registry::CommandRegistry;
use crate::modes::slash_dispatcher::{self, SlashAction, SlashRequest};
use crate::utils::get_nami_dir;
use adk_rust::Agent;
use adk_rust::Launcher;
use adk_rust::Llm;
use adk_session::SessionService;
use std::sync::Arc;
use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use rust_embed::RustEmbed;
use tower_http::cors::CorsLayer;
use serde_json::json;

#[derive(RustEmbed)]
#[folder = "webui/dist/"]
struct Asset;

pub async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    session: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    artifacts: Arc<dyn adk_artifact::ArtifactService>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::HeaderName::from_static("x-api-key")]);

    let app = Launcher::new(agent)
        .app_name("serve")
        .with_compaction(get_compaction_config(model))
        .with_session_service(session)
        .with_memory_service(memory)
        .with_artifact_service(artifacts)
        .with_a2a_base_url(base_url)
        .build_app()?
        .merge(crate::modes::api::api_router())
        .merge(Router::new().route("/.well-known/agent-card.json", get(|| async {
            Redirect::temporary("/.well-known/agent.json")
        })))
        .fallback(static_handler)
        .layer(middleware::from_fn(intercept_ui))
        .layer(middleware::from_fn(command_middleware))
        .layer(middleware::from_fn(stats_middleware))
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("\n==================================================");
    println!(" 🚀 Nami ADK Server starting on http://{}", addr);
    println!(" 📱 WebUI available at: http://{}", addr);
    println!(" 🔒 API Endpoints root:  http://{}/api", addr);
    println!("==================================================");
    println!(" Press Ctrl+C to stop\n");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
            println!("\nShutting down server gracefully...");
        })
        .await?;
    Ok(())
}

pub async fn intercept_ui(req: Request, next: Next) -> impl IntoResponse {
    let path = req.uri().path();
    if path == "/" || path == "/ui" || path.starts_with("/ui/") {
        static_handler(req.uri().clone()).await.into_response()
    } else {
        next.run(req).await
    }
}

pub async fn stats_middleware(req: Request, next: Next) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let is_agent_run = path.contains("/api/run/");

    if is_agent_run {
        let start_time = std::time::Instant::now();
        
        let content_length = req.headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let response = next.run(req).await;
        
        let duration_secs = start_time.elapsed().as_secs_f64();
        
        let resp_length = response.headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
            
        let final_resp_len = if resp_length > 0 { resp_length } else { 800 };

        let prompt_tokens = (content_length as f64 / 4.0).round() as usize;
        let response_tokens = (final_resp_len as f64 / 4.0).round() as usize;
        let total_tokens = prompt_tokens + response_tokens;
        
        let config = crate::agent::load_config_sync().ok();
        let provider = config.as_ref().and_then(|c| c.model.provider.clone()).unwrap_or_else(|| "unknown".to_string());
        let model_name = config.as_ref().map(|c| c.model.model_name.clone()).unwrap_or_else(|| "unknown".to_string());
        
        crate::utils::save_agent_statistic(&provider, &model_name, duration_secs, total_tokens);
        
        response
    } else {
        next.run(req).await
    }
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    // Clean up /ui prefix if present to match embedded asset paths
    if path.starts_with("ui/") {
        path = path.trim_start_matches("ui/").to_string();
    } else if path == "ui" {
        path = "".to_string();
    }

    if path.is_empty() || path == "index.html" {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            // Fallback to index.html for SPA routing
            if let Some(content) = Asset::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data))
                    .unwrap()
            } else {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }
    }
}

pub async fn command_middleware(req: Request, next: Next) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let is_run_path = path.contains("/api/run/") || path.contains("/run_sse");
    if !is_run_path {
        return next.run(req).await;
    }

    let (parts, body) = req.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 1024 * 1024).await else {
        return next.run(Request::from_parts(parts, Body::empty())).await;
    };

    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        let body = Body::from(bytes);
        return next.run(Request::from_parts(parts, body)).await;
    };

    let text = value
        .get("new_message")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let Some(text) = text else {
        let body = Body::from(bytes);
        return next.run(Request::from_parts(parts, body)).await;
    };

    if !text.starts_with('/') {
        let body = Body::from(bytes);
        return next.run(Request::from_parts(parts, body)).await;
    }

    let cmd_parts: Vec<&str> = text.splitn(2, ' ').collect();
    let command = cmd_parts[0];
    let args = cmd_parts.get(1).copied().unwrap_or("");

    let config_path = get_nami_dir().join("config.toml");
    let registry =
        CommandRegistry::load_from_config(&config_path.to_string_lossy()).unwrap_or_default();

    match slash_dispatcher::dispatch(SlashRequest {
        command,
        args,
        registry: &registry,
    }) {
        SlashAction::RunPrompt(prompt) => {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("new_message".to_string(), json!(prompt));
            }
            let new_body = Body::from(serde_json::to_vec(&value).unwrap_or(bytes.to_vec()));
            next.run(Request::from_parts(parts, new_body)).await
        }
        SlashAction::Reply(reply) => {
            (StatusCode::OK, Json(json!({ "response": reply }))).into_response()
        }
        SlashAction::PassThrough => {
            let body = Body::from(bytes);
            next.run(Request::from_parts(parts, body)).await
        }
    }
}
