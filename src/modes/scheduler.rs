use crate::agent::get_compaction_config;
use adk_rust::prelude::*;
use adk_rust::Agent;
use adk_rust::runner::Runner;
use adk_session::{CreateRequest, GetRequest, SessionService};
use crate::tools::scheduler::{load_schedule, save_schedule};
use crate::tools::state_manager::{load_states, TaskStatus};
use chrono::Utc;
use futures::StreamExt;
use std::sync::Arc;

pub async fn run_scheduler_loop(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    let app_name = "scheduler";
    let user_id = "system";
    let session_id = "background_tasks";

    ensure_session(&sessions, app_name, user_id, session_id).await?;

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions)
        .compaction_config(get_compaction_config(model))
        .build()?;

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        let mut tasks = match load_schedule().await {
            Ok(t) => t,
            Err(_) => continue,
        };

        let now = Utc::now();
        let mut changed = false;

        for task in tasks.iter_mut() {
            if !task.is_active {
                continue;
            }

            let schedule = match <cron::Schedule as std::str::FromStr>::from_str(&task.cron_expr) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let should_run = match task.last_run {
                Some(last) => {
                    if let Some(due) = schedule.after(&last).next() {
                        now >= due
                    } else {
                        false
                    }
                }
                None => true,
            };

            if should_run {
                let states = load_states().await.unwrap_or_default();
                let current_status = states
                    .iter()
                    .find(|s| s.goal == task.goal)
                    .map(|s| s.status.clone())
                    .unwrap_or(TaskStatus::InProgress);

                if current_status != TaskStatus::Completed {
                    log::info!("Scheduler triggering task: {}", task.goal);

                    let content = Content::new("user").with_text(&format!(
                        "SCHEDULED RUN: {}. Please continue working on this goal.",
                        task.goal
                    ));

                    let mut stream = runner.run_str(user_id, session_id, content).await?;
                    while let Some(_) = stream.next().await {}

                    task.last_run = Some(now);
                    changed = true;
                }
            }
        }

        if changed {
            let _ = save_schedule(&tasks).await;
        }
    }
}

async fn ensure_session(
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    if sessions
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            after: None,
            num_recent_events: None,
        })
        .await
        .is_err()
    {
        sessions
            .create(CreateRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: Some(session_id.to_string()),
                state: std::collections::HashMap::new(),
            })
            .await?;
    }
    Ok(())
}
