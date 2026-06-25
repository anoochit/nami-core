use crate::agent::get_compaction_config;
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
    Router,
};
use rust_embed::RustEmbed;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "webui/dist/"]
struct Asset;

pub async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    session: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
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
        .with_a2a_base_url(base_url)
        .build_app()?
        .merge(crate::modes::api::api_router())
        .merge(Router::new().route("/.well-known/agent-card.json", get(|| async {
            Redirect::temporary("/.well-known/agent.json")
        })))
        .fallback(static_handler)
        .layer(middleware::from_fn(intercept_ui))
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
