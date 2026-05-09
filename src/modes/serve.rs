use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::Launcher;
use adk_rust::Llm;
use std::sync::Arc;

pub(crate) async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    port: u16,
) -> anyhow::Result<()> {
    let base_url =
        std::env::var("A2A_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    Launcher::new(agent)
        .with_compaction(get_compaction_config(model))
        .with_a2a_base_url(base_url)
        .run_serve_directly(port)
        .await?;
    Ok(())
}
