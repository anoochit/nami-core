use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::Launcher;
use adk_rust::Llm;
use adk_session::SessionService;
use std::sync::Arc;
use axum::http::{header, Method};
use tower_http::cors::CorsLayer;
use axum::{routing::get, response::Redirect, Router};

pub async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    session:  Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
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
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("ADK Server starting on http://{}", addr);
    println!("Press Ctrl+C to stop\n");
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
            println!("\nShutting down server...");
        })
        .await?;
    Ok(())
}
