use crate::agent::get_compaction_config;
use adk_rust::{Agent, Launcher, Llm};
use axum::{
    body::Body,
    extract::Request,
    http::{header, Response, StatusCode, Uri, Method},
    middleware::{self, Next},
    response::IntoResponse,
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "webui/dist/"]
struct Asset;

pub async fn run_browse(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    memory: Arc<dyn adk_rust::Memory>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));

    log::info!("Starting Browse mode on port {}", port);
    log::info!("Embedding WebUI from webui/dist/");

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::HeaderName::from_static("x-api-key")]);

    // build_app() returns a router with hardcoded / -> /ui redirect and /ui routes.
    // We intercept these requests using middleware BEFORE they reach the routing table.
    let app = Launcher::new(agent)
        .app_name("webui")
        .with_compaction(get_compaction_config(model))
        .with_memory_service(memory)
        .with_a2a_base_url(base_url)
        .build_app()?
        .merge(crate::modes::api::api_router())
        .fallback(static_handler)
        .layer(middleware::from_fn(intercept_ui))
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("\nADK Server starting on http://{}", addr);
    println!("Open http://{} in your browser to access the UI", addr);
    println!("Press Ctrl+C to stop\n");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
            println!("\nShutting down server...");
        })
        .await?;

    Ok(())
}

async fn intercept_ui(req: Request, next: Next) -> impl IntoResponse {
    let path = req.uri().path();
    if path == "/" || path == "/ui" || path.starts_with("/ui/") {
        static_handler(req.uri().clone()).await.into_response()
    } else {
        next.run(req).await
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
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
