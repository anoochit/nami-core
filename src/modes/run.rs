use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::SessionService;
use crate::utils::session;
use futures::StreamExt;
use std::sync::Arc;

pub async fn run_direct(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    artifacts: Arc<dyn adk_artifact::ArtifactService>,
    model: Arc<dyn Llm>,
    provider: String,
    model_name: String,
    prompt: &str,
) -> anyhow::Result<()> {
    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "run_session";

    session::ensure_session(&sessions, app_name, user_id, session_id).await?;

    let compaction_cfg = crate::agent::load_config_sync().ok().and_then(|c| c.compaction);
    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions)
        .artifact_service(artifacts)
        .compaction_config(crate::agent::get_compaction_config(model.clone(), &compaction_cfg))
        .intra_compaction_config(crate::agent::get_intra_compaction_config(&model_name, &compaction_cfg))
        .intra_compaction_summarizer(crate::agent::get_intra_compaction_summarizer(model, &compaction_cfg))
        .build()?;

    let user_content = Content::new("user").with_text(prompt);

    let start_time = std::time::Instant::now();
    let mut stream = runner.run_str(user_id, session_id, user_content).await?;
    let mut full_response = String::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Some(content) = &event.llm_response.content {
                    for part in &content.parts {
                        if let Some(text) = &part.text() {
                            print!("{}", text);
                            std::io::Write::flush(&mut std::io::stdout())?;
                            full_response.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error running agent: {:?}", e);
                break;
            }
        }
    }
    println!();

    let duration_secs = start_time.elapsed().as_secs_f64();
    let prompt_tokens = (prompt.len() as f64 / 4.0).round() as usize;
    let response_tokens = (full_response.len() as f64 / 4.0).round() as usize;
    let total_tokens = prompt_tokens + response_tokens;

    // Save statistics to .nami/stats.json
    crate::utils::save_agent_statistic(&provider, &model_name, duration_secs, total_tokens);

    Ok(())
}
