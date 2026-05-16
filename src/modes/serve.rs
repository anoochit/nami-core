use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::Launcher;
use adk_rust::Llm;
use adk_session::SessionService;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::{routing::get, response::Redirect, Router};

pub(crate) async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    session:  Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

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
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("ADK Server starting on http://{}", addr);
    println!("Press Ctrl+C to stop\n");
    axum::serve(listener, app).await?;
    Ok(())
}
