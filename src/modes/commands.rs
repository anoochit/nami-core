use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::Utc;
use sqlx::{SqlitePool, Row};
use crate::utils::get_nami_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandState {
    Idle,
    Grilling {
        goal: String,
        questions: Vec<String>,
        answers: Vec<String>,
        current_index: usize,
    },
    Planning {
        plan_content: String,
        steps: Vec<String>,
    },
}

pub struct CommandStateManager {
    pool: SqlitePool,
}

impl CommandStateManager {
    pub async fn new() -> Result<Self> {
        let db_path = get_nami_dir().join("sessions.db");
        let url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await?;

        // Initialize command states table if not exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS nami_command_states (
                session_id TEXT PRIMARY KEY,
                state_json TEXT NOT NULL,
                updated_at DATETIME NOT NULL
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn get_state(&self, session_id: &str) -> Result<CommandState> {
        let row = sqlx::query("SELECT state_json FROM nami_command_states WHERE session_id = ?")
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let state_json: String = r.get(0);
            let state: CommandState = serde_json::from_str(&state_json)?;
            Ok(state)
        } else {
            Ok(CommandState::Idle)
        }
    }

    pub async fn set_state(&self, session_id: &str, state: &CommandState) -> Result<()> {
        let state_json = serde_json::to_string(state)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO nami_command_states (session_id, state_json, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(session_id) DO UPDATE SET state_json = excluded.state_json, updated_at = excluded.updated_at"
        )
        .bind(session_id)
        .bind(state_json)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear_state(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM nami_command_states WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Dynamic utility to parse steps from synthesized implementation plans.
pub fn parse_plan_steps(plan: &str) -> Vec<String> {
    let mut steps = Vec::new();
    let mut in_steps_section = false;

    for line in plan.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        if trimmed.starts_with('#') && (lower.contains("step") || lower.contains("plan")) {
            in_steps_section = true;
            continue;
        }

        if in_steps_section && trimmed.starts_with('#') {
            in_steps_section = false;
        }

        if in_steps_section || steps.is_empty() {
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- []") || trimmed.starts_with("* [ ]") {
                let step = trimmed[5..].trim().to_string();
                if !step.is_empty() {
                    steps.push(step);
                }
            } else if trimmed.starts_with("-") || trimmed.starts_with("*") {
                let step = trimmed[1..].trim().to_string();
                if !step.is_empty() && (step.to_lowercase().starts_with("step") || steps.len() < 10) {
                    steps.push(step);
                }
            }
        }
    }

    if steps.is_empty() {
        for line in plan.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ]") {
                let step = trimmed[5..].trim().to_string();
                if !step.is_empty() {
                    steps.push(step);
                }
            }
        }
    }

    steps
}
