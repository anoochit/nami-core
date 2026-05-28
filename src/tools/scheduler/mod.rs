use crate::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::{Deserialize, Serialize};
use adk_rust::tool::ToolContext;
use adk_tool::AdkError;
use chrono::{DateTime, Utc};
use cron::Schedule;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ScheduledTask {
    pub id: String,
    pub goal: String,
    pub cron_expr: String,
    pub last_run: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Deserialize, JsonSchema)]
struct ScheduleTaskArgs {
    /// Unique identifier for the schedule.
    pub id: String,
    /// The goal to execute.
    pub goal: String,
    /// Standard cron expression (e.g., '0 * * * * *' for every minute).
    pub cron_expr: String,
}

#[derive(Deserialize, JsonSchema)]
struct RemoveScheduleArgs {
    /// The ID of the schedule to remove.
    pub id: String,
}

async fn get_scheduler_file() -> std::result::Result<PathBuf, AdkError> {
    let root = get_workspace_dir().await?;
    Ok(root.join("scheduler.json"))
}

pub async fn load_schedule() -> std::result::Result<Vec<ScheduledTask>, AdkError> {
    let path = get_scheduler_file().await?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read scheduler: {}", e)))?;
    serde_json::from_str(&content)
        .map_err(|e| AdkError::tool(format!("Failed to parse scheduler: {}", e)))
}

pub async fn save_schedule(tasks: &[ScheduledTask]) -> std::result::Result<(), AdkError> {
    let path = get_scheduler_file().await?;
    let content = serde_json::to_string_pretty(tasks)
        .map_err(|e| AdkError::tool(format!("Failed to serialize scheduler: {}", e)))?;
    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write scheduler: {}", e)))
}

// --- Tools Implementation ---

pub struct ScheduleTask;
#[async_trait::async_trait]
impl Tool for ScheduleTask {
    fn name(&self) -> &str {
        "schedule_task"
    }
    fn description(&self) -> &str {
        "Schedules a task to run automatically using a cron expression."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Unique identifier for the schedule." },
                "goal": { "type": "string", "description": "The goal to execute." },
                "cron_expr": { "type": "string", "description": "Standard cron expression (e.g., '0 * * * * *' for every minute)." }
            },
            "required": ["id", "goal", "cron_expr"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: ScheduleTaskArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        // Validate cron expression
        let _ = Schedule::from_str(&args.cron_expr)
            .map_err(|e| AdkError::tool(format!("Invalid cron expression: {}", e)))?;

        let mut tasks = load_schedule().await?;
        if tasks.iter().any(|t| t.id == args.id) {
            return Err(AdkError::tool(format!(
                "Schedule ID '{}' already exists",
                args.id
            )));
        }

        tasks.push(ScheduledTask {
            id: args.id.clone(),
            goal: args.goal,
            cron_expr: args.cron_expr,
            last_run: None,
            is_active: true,
        });

        save_schedule(&tasks).await?;
        Ok(json!({"status": "success", "message": format!("Task '{}' scheduled", args.id)}))
    }
}

pub struct ListSchedule;
#[async_trait::async_trait]
impl Tool for ListSchedule {
    fn name(&self) -> &str {
        "list_schedule"
    }
    fn description(&self) -> &str {
        "Lists all active scheduled tasks."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({ "type": "object", "properties": {} }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        _args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let tasks = load_schedule().await?;
        if tasks.is_empty() {
            Ok(json!({"message": "No scheduled tasks found."}))
        } else {
            Ok(json!({ "scheduled_tasks": tasks }))
        }
    }
}

pub struct RemoveSchedule;
#[async_trait::async_trait]
impl Tool for RemoveSchedule {
    fn name(&self) -> &str {
        "remove_schedule"
    }
    fn description(&self) -> &str {
        "Removes a scheduled task."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The ID of the schedule to remove." }
            },
            "required": ["id"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: RemoveScheduleArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let mut tasks = load_schedule().await?;
        let original_len = tasks.len();
        tasks.retain(|t| t.id != args.id);

        if tasks.len() == original_len {
            return Err(AdkError::tool(format!(
                "Schedule ID '{}' not found",
                args.id
            )));
        }

        save_schedule(&tasks).await?;
        Ok(json!({"status": "success", "message": format!("Schedule '{}' removed", args.id)}))
    }
}

pub fn scheduler_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ScheduleTask),
        Arc::new(ListSchedule),
        Arc::new(RemoveSchedule),
    ]
}

#[cfg(test)]
mod test;
