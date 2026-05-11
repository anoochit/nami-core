use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::Launcher;
use adk_rust::Llm;
use std::sync::Arc;

pub(crate) async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    memory: Arc<dyn adk_rust::Memory>,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    let app = Launcher::new(agent)
        .with_compaction(get_compaction_config(model))
        .with_memory_service(memory)
        .with_a2a_base_url(base_url)
        .build_app()?
        .merge(crate::modes::api::api_router());

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("ADK Server starting on http://localhost:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}
