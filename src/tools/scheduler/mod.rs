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
    use super::*;
    use adk_rust::Tool;
    use adk_tool::SimpleToolContext;
    use std::sync::Arc;
    use tokio::fs;

    // Helper to safely run tests with workspace file isolation/backup.
    async fn setup_and_teardown<F, Fut>(test_fn: F)
    where
        F: FnOnce(PathBuf) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let path = get_scheduler_file().await.unwrap();
        let backup = if path.exists() {
            let content = fs::read_to_string(&path).await.unwrap();
            Some(content)
        } else {
            None
        };

        // Ensure we start with a clean state (no schedule file)
        if path.exists() {
            let _ = fs::remove_file(&path).await;
        }

        // Run the test, catching panics/errors to ensure cleanup happens
        let result = tokio::spawn(test_fn(path.clone())).await;

        // Cleanup and restore backup
        if path.exists() {
            let _ = fs::remove_file(&path).await;
        }
        if let Some(content) = backup {
            let _ = fs::write(&path, content).await;
        }

        if let Err(e) = result {
            std::panic::resume_unwind(e.into_panic());
        }
    }

    #[tokio::test]
    async fn test_scheduler_workflow() {
        setup_and_teardown(|_path| async move {
            let ctx = Arc::new(SimpleToolContext::new("test_caller"));
            let schedule_tool = ScheduleTask;
            let list_tool = ListSchedule;
            let remove_tool = RemoveSchedule;

            // 1. Initially ListSchedule should say "No scheduled tasks found."
            let list_res = list_tool.execute(ctx.clone(), json!({})).await.unwrap();
            assert_eq!(list_res["message"], "No scheduled tasks found.");

            // 2. Schedule a valid task
            let schedule_args = json!({
                "id": "task_1",
                "goal": "Test scheduling logic",
                "cron_expr": "0 0 * * * *"
            });
            let sched_res = schedule_tool.execute(ctx.clone(), schedule_args).await.unwrap();
            assert_eq!(sched_res["status"], "success");
            assert!(sched_res["message"].as_str().unwrap().contains("task_1"));

            // 3. Scheduling duplicate ID should fail
            let schedule_dup_args = json!({
                "id": "task_1",
                "goal": "Another goal",
                "cron_expr": "0 0 * * * *"
            });
            let sched_dup_res = schedule_tool.execute(ctx.clone(), schedule_dup_args).await;
            assert!(sched_dup_res.is_err());
            let err_msg = sched_dup_res.unwrap_err().to_string();
            assert!(err_msg.contains("already exists"));

            // 4. Scheduling with invalid cron expression should fail
            let schedule_invalid_cron = json!({
                "id": "task_2",
                "goal": "Invalid cron task",
                "cron_expr": "invalid cron expr"
            });
            let sched_invalid_res = schedule_tool.execute(ctx.clone(), schedule_invalid_cron).await;
            assert!(sched_invalid_res.is_err());

            // 5. ListSchedule should return our scheduled task
            let list_res2 = list_tool.execute(ctx.clone(), json!({})).await.unwrap();
            let tasks = &list_res2["scheduled_tasks"];
            assert!(tasks.is_array());
            assert_eq!(tasks.as_array().unwrap().len(), 1);
            assert_eq!(tasks[0]["id"], "task_1");
            assert_eq!(tasks[0]["goal"], "Test scheduling logic");
            assert_eq!(tasks[0]["cron_expr"], "0 0 * * * *");
            assert_eq!(tasks[0]["is_active"], true);

            // 6. Remove the scheduled task
            let remove_args = json!({
                "id": "task_1"
            });
            let remove_res = remove_tool.execute(ctx.clone(), remove_args).await.unwrap();
            assert_eq!(remove_res["status"], "success");

            // 7. Removing nonexistent task should fail
            let remove_nonexistent_args = json!({
                "id": "task_1"
            });
            let remove_nonexistent_res = remove_tool.execute(ctx.clone(), remove_nonexistent_args).await;
            assert!(remove_nonexistent_res.is_err());

            // 8. ListSchedule should be empty again
            let list_res3 = list_tool.execute(ctx.clone(), json!({})).await.unwrap();
            assert_eq!(list_res3["message"], "No scheduled tasks found.");
        }).await;
    }
}
