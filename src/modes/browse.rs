use crate::agent::get_compaction_config;
use adk_rust::{Agent, Launcher, Llm};
use adk_session::SessionService;
use axum::{
    body::Body,
    extract::Request,
    http::{header, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::IntoResponse,
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(RustEmbed)]
#[folder = "webui/dist/"]
struct Asset;

pub(crate) async fn run_browse(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    session:  Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    log::info!("Starting Browse mode on port {}", port);
    log::info!("Embedding WebUI from webui/dist/");

    // build_app() returns a router with hardcoded / -> /ui redirect and /ui routes.
    // We intercept these requests using middleware BEFORE they reach the routing table.
    let app = Launcher::new(agent)
        .app_name("webui")
        .with_compaction(get_compaction_config(model))
        .with_session_service(session)
        .with_memory_service(memory)
        .with_a2a_base_url(base_url)
        .build_app()?
        .merge(crate::modes::api::api_router())
        .fallback(static_handler)
        .layer(middleware::from_fn(intercept_ui))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("\nADK Server starting on http://{}", addr);
    println!("Open http://{} in your browser to access the UI", addr);
    println!("Press Ctrl+C to stop\n");
    axum::serve(listener, app).await?;

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
