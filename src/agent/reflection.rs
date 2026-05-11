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

const COMPACTION_THRESHOLD: usize = 80; // compact when > 80 bullet points

#[derive(Debug, Serialize, Deserialize, Default)]
struct ReflectionState {
    last_processed_timestamp: Option<DateTime<Utc>>,
    insight_count: usize,
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
        Self { model, model_name, memory }
    }

    pub async fn start(self: Arc<Self>) {
        log::info!("Reflection Service started.");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 30));
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
            serde_json::from_str(&fs::read_to_string(&state_path).await?).unwrap_or_default()
        } else {
            ReflectionState::default()
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite:sessions.db?mode=ro")
            .await?;

        // --- 1. Fetch active sessions (parameterized) ---
        let active_sessions: Vec<(String, String, String)> = if let Some(ref ts) = state.last_processed_timestamp {
            sqlx::query(
                "SELECT DISTINCT app_name, user_id, session_id FROM events \
                 WHERE timestamp > ? AND author != 'nami' AND author != 'reflection_service'"
            )
            .bind(ts.to_rfc3339())
            .map(|row: SqliteRow| (row.get(0), row.get(1), row.get(2)))
            .fetch_all(&pool)
            .await?
        } else {
            sqlx::query(
                "SELECT DISTINCT app_name, user_id, session_id FROM events \
                 WHERE author != 'nami' AND author != 'reflection_service'"
            )
            .map(|row: SqliteRow| (row.get(0), row.get(1), row.get(2)))
            .fetch_all(&pool)
            .await?
        };

        if active_sessions.is_empty() {
            log::info!("No new session activity.");
            return Ok(());
        }

        // --- 2. Collect all logs across sessions ---
        let mut all_logs = String::new();
        let mut latest_ts = state.last_processed_timestamp.clone();

        for (app, user, sid) in active_sessions {
            let rows: Vec<SqliteRow> = if let Some(ref ts) = state.last_processed_timestamp {
                sqlx::query(
                    "SELECT author, timestamp, llm_response FROM events \
                     WHERE app_name = ? AND user_id = ? AND session_id = ? AND timestamp > ? \
                     ORDER BY timestamp ASC"
                )
                .bind(&app).bind(&user).bind(&sid).bind(ts.to_rfc3339())
                .fetch_all(&pool).await?
            } else {
                sqlx::query(
                    "SELECT author, timestamp, llm_response FROM events \
                     WHERE app_name = ? AND user_id = ? AND session_id = ? \
                     ORDER BY timestamp ASC"
                )
                .bind(&app).bind(&user).bind(&sid)
                .fetch_all(&pool).await?
            };

            if rows.is_empty() { continue; }

            all_logs.push_str(&format!("\n--- Session: {}/{}/{} ---\n", app, user, sid));

            for row in rows {
                let author: String = row.get(0);
                let ts_str: String = row.get(1);
                let llm_json: String = row.get(2);

                if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                    if latest_ts.as_ref().map_or(true, |lts| &ts > lts) {
                        latest_ts = Some(ts);
                    }
                }

                let role = if author == "nami" { "Nami" } else { "User" };
                let text = extract_text_from_event(&llm_json);
                if !text.is_empty() {
                    all_logs.push_str(&format!("{}: {}\n", role, text));
                }
            }
        }

        if all_logs.trim().is_empty() {
            return Ok(());
        }

        // --- 3. Single synthesis pass over all new logs ---
        let current_memories = fs::read_to_string(&memories_path).await.unwrap_or_default();

        let extract_prompt = format!(
            r#"You are a Memory Architect. Extract NEW, permanent facts from the conversation logs below.

Rules:
- Include: personal facts, preferences, long-term context, stated goals, relationships, locations.
- Exclude: transient requests, already-known facts (see existing memories), filler conversation.
- If a new fact CONTRADICTS an existing memory, prefix with "CORRECTION: " and state the update.
- If there's nothing new, return exactly: NONE

Existing Memories:
{}

New Conversation Logs:
{}

Return a bulleted list, one item per line starting with "- ". No extra commentary."#,
            current_memories, all_logs
        );

        let raw_insights = self.llm_complete(&extract_prompt).await?;
        if raw_insights.trim() == "NONE" || raw_insights.trim().is_empty() {
            log::info!("No new insights this cycle.");
            self.save_state(&state_path, &mut state, latest_ts).await?;
            return Ok(());
        }

        // --- 4. Deduplication + contradiction resolution pass ---
        let dedup_prompt = format!(
            r#"You are a Memory Curator. Given the existing memories and a list of candidate new insights, produce a clean final memory file.

Rules:
- Remove duplicates (same fact stated differently).
- For contradictions: keep only the NEWER fact (marked "CORRECTION:"), drop the old one.
- Merge closely related facts into single, precise bullet points.
- Preserve all existing memories that are NOT superseded.
- Return the COMPLETE updated memory list as bullet points, nothing else.

Existing Memories:
{}

Candidate New Insights:
{}"#,
            current_memories, raw_insights
        );

        let merged_memories = self.llm_complete(&dedup_prompt).await?;
        let bullet_count = merged_memories.lines().filter(|l| l.trim_start().starts_with("- ")).count();

        // --- 5. Compact if growing too large ---
        let final_memories = if bullet_count > COMPACTION_THRESHOLD {
            log::info!("Memory file has {} entries, compacting...", bullet_count);
            self.compact_memories(&merged_memories).await?
        } else {
            merged_memories
        };

        // --- 6. Write memories (single source of truth = MEMORIES.md) ---
        let header = "# MEMORIES\n\n";
        fs::write(&memories_path, format!("{}{}\n", header, final_memories.trim())).await?;

        // Mirror to searchable memory service
        let entry = MemoryEntry {
            content: Content::new("user").with_text(&final_memories),
            author: "reflection_service".into(),
            timestamp: Utc::now(),
        };
        self.memory.add_session("nami", "system", "reflection_learnings", vec![entry]).await?;

        log::info!("Reflection cycle complete. {} memory entries.", bullet_count);

        state.insight_count = bullet_count;
        self.save_state(&state_path, &mut state, latest_ts).await?;
        Ok(())
    }

    async fn compact_memories(&self, memories: &str) -> anyhow::Result<String> {
        let prompt = format!(
            r#"You are a Memory Compactor. The memory list below has grown large. 
Consolidate it by:
- Merging related facts into concise composite bullets.
- Dropping facts that are obviously superseded or trivially implied by others.
- Preserving ALL unique, meaningful information.
- Target: under 50 bullet points without losing substance.

Memories to compact:
{}

Return only the compacted bullet list."#,
            memories
        );
        self.llm_complete(&prompt).await
    }

    async fn llm_complete(&self, prompt: &str) -> anyhow::Result<String> {
        let mut stream = self.model.generate_content(
            LlmRequest::new(
                self.model_name.clone(),
                vec![Content::new("user").with_text(prompt)],
            ),
            false,
        ).await?;

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
        Ok(text.trim().to_string())
    }

    async fn save_state(
        &self,
        path: &std::path::Path,
        state: &mut ReflectionState,
        latest_ts: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        state.last_processed_timestamp = latest_ts;
        fs::write(path, serde_json::to_string_pretty(&state)?).await?;
        Ok(())
    }
}

/// Robustly extract text from an event's llm_json field.
/// Handles both `content.parts[].text` and flat `text` shapes.
fn extract_text_from_event(llm_json: &str) -> String {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(llm_json) else {
        return String::new();
    };

    // Shape 1: content.parts[].text
    if let Some(parts) = val.get("content")
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
    {
        let text: String = parts.iter()
            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        if !text.is_empty() { return text; }
    }

    // Shape 2: flat top-level text field
    if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
        if !text.is_empty() { return text.to_string(); }
    }

    // Shape 3: content as plain string
    if let Some(text) = val.get("content").and_then(|c| c.as_str()) {
        if !text.is_empty() { return text.to_string(); }
    }

    String::new()
}