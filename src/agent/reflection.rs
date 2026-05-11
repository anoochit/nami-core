use adk_rust::prelude::*;
use adk_session::SessionService;
use adk_memory::{MemoryEntry, MemoryService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;
use crate::utils::get_workspace_dir;
use sqlx::sqlite::{SqlitePoolOptions, SqliteRow};
use sqlx::Row;
use chrono::{Utc, DateTime};
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ReflectionState {
    last_processed_timestamp: Option<DateTime<Utc>>,
}

pub struct ReflectionService {
    model: Arc<dyn Llm>,
    model_name: String,
    memory: Arc<dyn MemoryService>,
}

impl ReflectionService {
    pub fn new(
        model: Arc<dyn Llm>,
        model_name: String,
        _sessions: Arc<dyn SessionService>,
        memory: Arc<dyn MemoryService>,
    ) -> Self {
        Self {
            model,
            model_name,
            memory,
        }
    }

    pub async fn start(self: Arc<Self>) {
        log::info!("Agent Reflection Service started.");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 30)); // Every 30 minutes
        loop {
            interval.tick().await;
            if let Err(e) = self.run_cycle().await {
                log::error!("Reflection cycle failed: {:?}", e);
            }
        }
    }

    async fn run_cycle(&self) -> anyhow::Result<()> {
        log::info!("Running reflection cycle...");

        let workspace = get_workspace_dir().await?;
        let state_path = workspace.join("reflection_state.json");
        let memories_path = workspace.join("MEMORIES.md");

        let mut state: ReflectionState = if state_path.exists() {
            let content = fs::read_to_string(&state_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            ReflectionState::default()
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite:sessions.db?mode=ro")
            .await?;

        // Find unique sessions with activity since last check
        let query = if let Some(ref ts) = state.last_processed_timestamp {
            format!("SELECT DISTINCT app_name, user_id, session_id FROM events WHERE timestamp > '{}' AND author != 'nami' AND author != 'reflection_service'", ts.to_rfc3339())
        } else {
            "SELECT DISTINCT app_name, user_id, session_id FROM events WHERE author != 'nami' AND author != 'reflection_service'".to_string()
        };

        let active_sessions: Vec<(String, String, String)> = sqlx::query(&query)
            .map(|row: SqliteRow| {
                (row.get(0), row.get(1), row.get(2))
            })
            .fetch_all(&pool)
            .await?;

        if active_sessions.is_empty() {
            log::info!("No new session activity since last reflection.");
            return Ok(());
        }

        let mut new_insights = Vec::new();
        let mut latest_ts = state.last_processed_timestamp;

        for (app, user, sid) in active_sessions {
            // Fetch messages for this session
            let event_query = if let Some(ref ts) = state.last_processed_timestamp {
                format!("SELECT author, timestamp, llm_response FROM events WHERE app_name = '{}' AND user_id = '{}' AND session_id = '{}' AND timestamp > '{}' ORDER BY timestamp ASC", app, user, sid, ts.to_rfc3339())
            } else {
                format!("SELECT author, timestamp, llm_response FROM events WHERE app_name = '{}' AND user_id = '{}' AND session_id = '{}' ORDER BY timestamp ASC", app, user, sid)
            };

            let rows = sqlx::query(&event_query).fetch_all(&pool).await?;
            
            let mut log_text = String::new();
            for row in rows {
                let author: String = row.get(0);
                let timestamp_str: String = row.get(1);
                let llm_json: String = row.get(2);

                let ts: DateTime<Utc> = timestamp_str.parse()?;
                if latest_ts.as_ref().map_or(true, |lts| &ts > lts) {
                    latest_ts = Some(ts);
                }

                let role = if author == "nami" { "Nami" } else { "User" };
                
                // Parse the JSON to get text
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&llm_json) {
                    if let Some(parts) = val.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                        let text = parts.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join(" ");
                        if !text.is_empty() {
                            log_text.push_str(&format!("{}: {}\n", role, text));
                        }
                    }
                }
            }

            if log_text.is_empty() { continue; }

            // Perform synthesis
            let current_memories = fs::read_to_string(&memories_path).await.unwrap_or_default();
            
            let prompt = format!(
                r#"You are a Memory Architect for Nami. Your goal is to extract NEW, permanent insights from the conversation logs below.
Insights should include:
- User's technical preferences (e.g. "prefers async Rust", "uses Tailwind").
- Project details (e.g. "working on 'namiclaw' project").
- Personal facts or context provided by the user.

Exclude:
- Transient info (e.g. "asked for a coffee recipe").
- Info already in the memory.
- Conversational filler.

Existing Memories:
{}

New Conversation Log:
{}

Return a bulleted list of NEW insights to remember, one per line starting with "- ". 
If there are no new permanent insights, return exactly "NONE"."#,
                current_memories, log_text
            );

            let mut stream = self.model.generate_content(LlmRequest::new(self.model_name.clone(), vec![Content::new("user").with_text(prompt)]), false).await?;
            let mut text = String::new();
            while let Some(event) = stream.next().await {
                let event = event?;
                if let Some(content) = event.content {
                    for part in content.parts {
                        if let Some(t) = part.text() {
                            text.push_str(t);
                        }
                    }
                }
            }
            let text = text.trim().to_string();

            if text != "NONE" && !text.is_empty() {
                new_insights.push(text);
            }
        }

        if !new_insights.is_empty() {
            let combined_insights = new_insights.join("\n");
            log::info!("Synthesized new insights:\n{}", combined_insights);

            // Update MEMORIES.md
            let mut memories = fs::read_to_string(&memories_path).await.unwrap_or_else(|_| "# MEMORIES\n".to_string());
            if !memories.ends_with('\n') { memories.push('\n'); }
            memories.push_str(&combined_insights);
            memories.push('\n');
            fs::write(&memories_path, memories).await?;

            // Update searchable memory
            let entry = MemoryEntry {
                content: Content::new("user").with_text(&format!("Synthesized learnings:\n{}", combined_insights)),
                author: "reflection_service".into(),
                timestamp: Utc::now(),
            };
            self.memory.add_session("nami", "system", "reflection_learnings", vec![entry]).await?;
        }

        // Save state
        state.last_processed_timestamp = latest_ts;
        fs::write(&state_path, serde_json::to_string_pretty(&state)?).await?;

        log::info!("Reflection cycle completed.");
        Ok(())
    }
}
