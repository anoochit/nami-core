use crate::agent::get_compaction_config;
use crate::utils::{categorize_error, ErrorCategory};
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

/// Manages the execution of agent-based tasks within a specific application context.
pub struct AgentRunner {
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    app_name: String,
    model: Arc<dyn Llm>,
}

impl AgentRunner {
    /// Creates a new `AgentRunner`.
    pub fn new(
        agent: Arc<dyn Agent>,
        sessions: Arc<dyn SessionService>,
        memory: Arc<dyn adk_rust::Memory>,
        app_name: impl Into<String>,
        model: Arc<dyn Llm>,
    ) -> Self {
        Self {
            agent,
            sessions,
            memory,
            app_name: app_name.into(),
            model,
        }
    }

    /// Executes a single input within a session, returning the agent's response.
    pub async fn run(
        &self,
        user_id: &str,
        session_id: &str,
        input: &str,
    ) -> anyhow::Result<String> {
        let mut retries = 3;
        let mut delay = Duration::from_secs(1);

        loop {
            match self.execute_once(user_id, session_id, input).await {
                Ok(response) => return Ok(response),
                Err(e) if retries > 0 && categorize_error(&e) == ErrorCategory::Transient => {
                    log::warn!("Transient error encountered (retries left: {}): {}", retries, e);
                    sleep(delay).await;
                    retries -= 1;
                    delay *= 2;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn execute_once(
        &self,
        user_id: &str,
        session_id: &str,
        input: &str,
    ) -> anyhow::Result<String> {
        // ensure session exists
        if self
            .sessions
            .get(GetRequest {
                app_name: self.app_name.clone(),
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                num_recent_events: None,
                after: None,
            })
            .await
            .is_err()
        {
            self.sessions
                .create(CreateRequest {
                    app_name: self.app_name.clone(),
                    user_id: user_id.to_string(),
                    session_id: Some(session_id.to_string()),
                    state: Default::default(),
                })
                .await?;
        }

        let runner = Runner::builder()
            .app_name(&self.app_name)
            .agent(self.agent.clone())
            .session_service(self.sessions.clone())
            .memory_service(self.memory.clone())
            .compaction_config(get_compaction_config(self.model.clone()))
            .build()?;

        let content = Content::new("user").with_text(input);

        let mut stream = runner.run_str(user_id, session_id, content).await?;

        let mut response = String::new();

        while let Some(result) = stream.next().await {
            let event = match result {
                Ok(event) => event,
                Err(e) => return Err(e.into()),
            };

            if let Some(content) = &event.llm_response.content {
                for part in &content.parts {
                    if let Some(text) = part.text() {
                        response.push_str(text);
                    }
                }
            }
        }

        if response.is_empty() {
            response = "Sorry, I couldn't generate a response.".to_string();
        }

        Ok(response)
    }
}
