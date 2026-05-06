use crate::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::{Deserialize, Serialize};
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    InProgress,
    Blocked,
    Completed,
    Failed,
}

impl std::str::FromStr for TaskStatus {
    type Err = AdkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "in_progress" => Ok(TaskStatus::InProgress),
            "blocked" => Ok(TaskStatus::Blocked),
            "completed" => Ok(TaskStatus::Completed),
            "failed" => Ok(TaskStatus::Failed),
            _ => Err(AdkError::tool(format!("Invalid status: {}", s))),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Step {
    pub description: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct TaskState {
    pub task_id: String,
    pub status: TaskStatus,
    pub goal: String,
    pub steps: Vec<Step>,
    pub last_step: Option<String>,
    pub context_payload: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, JsonSchema)]
struct InitTaskArgs {
    /// Unique identifier for the task.
    task_id: String,
    /// High-level objective of the task.
    goal: String,
    /// List of initial execution steps (descriptions).
    steps: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct StepUpdate {
    /// Description of the step.
    description: String,
    /// Whether the step is completed.
    completed: bool,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateTaskArgs {
    /// The ID of the task to update.
    task_id: String,
    /// New status: in_progress, blocked, completed, failed.
    status: Option<String>,
    /// Summary of the last completed action.
    last_step: Option<String>,
    /// Data needed for the next run (JSON object).
    context_payload: Option<Value>,
    /// Updated list of steps (replaces existing if provided).
    steps: Option<Vec<StepUpdate>>,
}

#[derive(Deserialize, JsonSchema)]
struct TaskIdArgs {
    /// The ID of the task.
    task_id: String,
}

async fn get_states_file() -> std::result::Result<PathBuf, AdkError> {
    let root: std::path::PathBuf = get_workspace_dir().await?;
    Ok(root.join("task_states.json"))
}

async fn load_states() -> std::result::Result<Vec<TaskState>, AdkError> {
    let path = get_states_file().await?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read task states: {}", e)))?;
    serde_json::from_str(&content)
        .map_err(|e| AdkError::tool(format!("Failed to parse task states: {}", e)))
}

async fn save_states(states: &[TaskState]) -> std::result::Result<(), AdkError> {
    let path = get_states_file().await?;
    let content = serde_json::to_string_pretty(states)
        .map_err(|e| AdkError::tool(format!("Failed to serialize task states: {}", e)))?;
    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write task states: {}", e)))
}

/// Initializes a new long-running task.
#[tool]
async fn init_task(args: InitTaskArgs) -> std::result::Result<Value, AdkError> {
    let mut states = load_states().await?;
    if states.iter().any(|t| t.task_id == args.task_id) {
        return Err(AdkError::tool(format!("Task with ID '{}' already exists", args.task_id)));
    }

    let steps = args.steps.into_iter().map(|d| Step { description: d, completed: false }).collect();
    
    let new_task = TaskState {
        task_id: args.task_id.clone(),
        status: TaskStatus::InProgress,
        goal: args.goal,
        steps,
        last_step: None,
        context_payload: json!({}),
        updated_at: Utc::now(),
    };

    states.push(new_task);
    save_states(&states).await?;
    Ok(json!({"status": "success", "message": format!("Task '{}' initialized", args.task_id)}))
}

/// Updates the state of an existing task.
#[tool]
async fn update_task(args: UpdateTaskArgs) -> std::result::Result<Value, AdkError> {
    let mut states = load_states().await?;
    if let Some(task) = states.iter_mut().find(|t| t.task_id == args.task_id) {
        if let Some(status_str) = args.status {
            task.status = status_str.parse()?;
        }
        if let Some(last_step) = args.last_step {
            task.last_step = Some(last_step);
        }
        if let Some(payload) = args.context_payload {
            task.context_payload = payload;
        }
        if let Some(steps) = args.steps {
            task.steps = steps.into_iter().map(|s| Step {
                description: s.description,
                completed: s.completed,
            }).collect();
        }
        task.updated_at = Utc::now();
        
        save_states(&states).await?;
        Ok(json!({"status": "success", "message": format!("Task '{}' updated", args.task_id)}))
    } else {
        Err(AdkError::tool(format!("Task '{}' not found", args.task_id)))
    }
}

/// Retrieves the current state of a specific task.
#[tool]
async fn get_task(args: TaskIdArgs) -> std::result::Result<Value, AdkError> {
    let states = load_states().await?;
    if let Some(task) = states.into_iter().find(|t| t.task_id == args.task_id) {
        Ok(json!(task))
    } else {
        Err(AdkError::tool(format!("Task '{}' not found", args.task_id)))
    }
}

/// Lists all tasks that are currently active (in_progress or blocked).
#[tool]
async fn list_active_tasks(_args: Value) -> std::result::Result<Value, AdkError> {
    let states = load_states().await?;
    let active: Vec<_> = states.into_iter()
        .filter(|t| matches!(t.status, TaskStatus::InProgress | TaskStatus::Blocked))
        .collect();
    
    if active.is_empty() {
        Ok(json!({"message": "No active tasks found."}))
    } else {
        Ok(json!({ "active_tasks": active }))
    }
}

pub fn state_manager_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(InitTask),
        Arc::new(UpdateTask),
        Arc::new(GetTask),
        Arc::new(ListActiveTasks),
    ]
}
