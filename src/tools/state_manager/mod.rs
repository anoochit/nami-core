use crate::utils::get_nami_dir;
use adk_rust::Tool;
use adk_rust::serde::{Deserialize, Serialize};
use adk_rust::tool::ToolContext;
use adk_tool::AdkError;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Blocked,
    #[serde(alias = "completed")]
    Done,
    #[serde(alias = "failed")]
    Cancelled,
}

impl TaskStatus {
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TaskStatus::Todo | TaskStatus::InProgress | TaskStatus::InReview | TaskStatus::Blocked
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = AdkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "backlog" => Ok(TaskStatus::Backlog),
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "in_review" => Ok(TaskStatus::InReview),
            "blocked" => Ok(TaskStatus::Blocked),
            "done" | "completed" => Ok(TaskStatus::Done),
            "cancelled" | "failed" | "cancel" => Ok(TaskStatus::Cancelled),
            _ => Err(AdkError::tool(format!("Invalid status: {}", s))),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VerificationMethod {
    Llm {
        criteria: String,
    },
    Command {
        command: String,
        expected_output: Option<String>,
        #[serde(default)]
        allow_failure: bool,
    },
    FileExists {
        path: String,
        contains_pattern: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepPhase {
    Design,
    Implementation,
    Integration,
    Verification,
}

impl std::fmt::Display for StepPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepPhase::Design => write!(f, "Design"),
            StepPhase::Implementation => write!(f, "Implementation"),
            StepPhase::Integration => write!(f, "Integration"),
            StepPhase::Verification => write!(f, "Verification"),
        }
    }
}

impl std::str::FromStr for StepPhase {
    type Err = AdkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "design" => Ok(StepPhase::Design),
            "implementation" | "implement" => Ok(StepPhase::Implementation),
            "integration" | "integrate" => Ok(StepPhase::Integration),
            "verification" | "verify" => Ok(StepPhase::Verification),
            _ => Err(AdkError::tool(format!("Invalid step phase: {}", s))),
        }
    }
}

pub fn default_phase() -> StepPhase {
    StepPhase::Implementation
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Step {
    pub description: String,
    pub completed: bool,
    pub verification_criteria: Option<String>,
    #[serde(default)]
    pub verification_method: Option<VerificationMethod>,
    #[serde(default = "default_phase")]
    pub phase: StepPhase,
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
    /// Initial status of the task.
    status: Option<String>,
    /// List of initial execution steps. Can be strings (description) or objects.
    steps: Vec<Value>,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateTaskArgs {
    /// The ID of the task to update.
    task_id: String,
    /// New status: backlog, todo, in_progress, in_review, blocked, done, cancelled.
    status: Option<String>,
    /// Summary of the last completed action.
    last_step: Option<String>,
    /// Data needed for the next run (JSON object).
    context_payload: Option<Value>,
    /// Updated list of steps as JSON array: [{"description": "...", "completed": bool, "verification_criteria": "..."}]
    steps: Option<Value>,
}

#[derive(Deserialize, JsonSchema)]
struct TaskIdArgs {
    /// The ID of the task.
    task_id: String,
}

async fn get_states_file() -> std::result::Result<PathBuf, AdkError> {
    Ok(get_nami_dir().join("task_states.json"))
}

async fn get_archive_file() -> std::result::Result<PathBuf, AdkError> {
    Ok(get_nami_dir().join("task_states_archive.json"))
}

pub async fn load_states() -> std::result::Result<Vec<TaskState>, AdkError> {
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

async fn archive_tasks(tasks_to_archive: &[TaskState]) -> std::result::Result<(), AdkError> {
    if tasks_to_archive.is_empty() {
        return Ok(());
    }
    let path = get_archive_file().await?;
    let mut archived = if path.exists() {
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to read task states archive: {}", e)))?;
        serde_json::from_str::<Vec<TaskState>>(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Prevent duplicate entries in archive
    for task in tasks_to_archive {
        if !archived.iter().any(|t| t.task_id == task.task_id && t.updated_at == task.updated_at) {
            archived.push(task.clone());
        }
    }

    let content = serde_json::to_string_pretty(&archived)
        .map_err(|e| AdkError::tool(format!("Failed to serialize task states archive: {}", e)))?;
    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write task states archive: {}", e)))
}

async fn save_states(states: &[TaskState]) -> std::result::Result<(), AdkError> {
    let mut active_states = Vec::new();
    let mut to_archive = Vec::new();

    // Auto-archive completed/cancelled tasks older than 5 minutes to keep main list clean
    for task in states {
        if task.status.is_terminal() && Utc::now().signed_duration_since(task.updated_at).num_minutes() >= 5 {
            to_archive.push(task.clone());
        } else {
            active_states.push(task.clone());
        }
    }

    if !to_archive.is_empty() {
        archive_tasks(&to_archive).await?;
    }

    let path = get_states_file().await?;
    let content = serde_json::to_string_pretty(&active_states)
        .map_err(|e| AdkError::tool(format!("Failed to serialize task states: {}", e)))?;
    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write task states: {}", e)))
}

// --- Tools Implementation ---

pub struct InitTask;
#[async_trait::async_trait]
impl Tool for InitTask {
    fn name(&self) -> &str {
        "init_task"
    }
    fn description(&self) -> &str {
        "Initializes a new long-running task."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "Unique identifier for the task." },
                "goal": { "type": "string", "description": "High-level objective of the task." },
                "status": {
                    "type": "string",
                    "enum": ["backlog", "todo", "in_progress", "in_review", "blocked", "done", "cancelled"],
                    "description": "Initial status of the task (defaults to in_progress)."
                },
                "steps": {
                    "type": "array",
                    "items": {
                        "oneOf": [
                            { "type": "string", "description": "Description of the step." },
                            {
                                "type": "object",
                                "properties": {
                                    "description": { "type": "string" },
                                    "phase": { "type": "string", "enum": ["design", "implementation", "integration", "verification"] },
                                    "verification_criteria": { "type": "string" },
                                    "verification_method": { "type": "object", "description": "Optional structured verification method definition." }
                                },
                                "required": ["description"]
                            }
                        ]
                    },
                    "description": "List of initial execution steps (descriptions or objects with criteria)."
                }
            },
            "required": ["task_id", "goal", "steps"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: InitTaskArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let mut states = load_states().await?;
        if states.iter().any(|t| t.task_id == args.task_id) {
            return Err(AdkError::tool(format!(
                "Task with ID '{}' already exists",
                args.task_id
            )));
        }

        let initial_status = if let Some(s) = args.status {
            s.parse()?
        } else {
            TaskStatus::InProgress
        };

        let steps = args
            .steps
            .into_iter()
            .map(|v| {
                if let Some(s) = v.as_str() {
                    Ok(Step {
                        description: s.to_string(),
                        completed: false,
                        verification_criteria: None,
                        verification_method: None,
                        phase: StepPhase::Implementation,
                    })
                } else {
                    #[derive(Deserialize)]
                    struct StepInput {
                        description: String,
                        #[serde(default = "default_phase")]
                        phase: StepPhase,
                        verification_criteria: Option<String>,
                        verification_method: Option<VerificationMethod>,
                    }
                    let input: StepInput = serde_json::from_value(v)
                        .map_err(|e| AdkError::tool(format!("Invalid step format: {}", e)))?;
                    Ok(Step {
                        description: input.description,
                        completed: false,
                        verification_criteria: input.verification_criteria,
                        verification_method: input.verification_method,
                        phase: input.phase,
                    })
                }
            })
            .collect::<std::result::Result<Vec<_>, AdkError>>()?;

        let new_task = TaskState {
            task_id: args.task_id.clone(),
            status: initial_status,
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
}

pub struct UpdateTask;
#[async_trait::async_trait]
impl Tool for UpdateTask {
    fn name(&self) -> &str {
        "update_task"
    }
    fn description(&self) -> &str {
        "Updates the state of an existing task."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The ID of the task to update." },
                "status": {
                    "type": "string",
                    "enum": ["backlog", "todo", "in_progress", "in_review", "blocked", "done", "cancelled"],
                    "description": "New Kanban status for the task."
                },
                "last_step": { "type": "string", "description": "Summary of the last completed action." },
                "context_payload": { "type": "object", "description": "Data needed for the next run." },
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": { "type": "string" },
                            "completed": { "type": "boolean" },
                            "phase": { "type": "string", "enum": ["design", "implementation", "integration", "verification"] },
                            "verification_criteria": { "type": "string" },
                            "verification_method": { "type": "object" }
                        },
                        "required": ["description"]
                    },
                    "description": "Updated list of steps."
                }
            },
            "required": ["task_id"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: UpdateTaskArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

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
            if let Some(steps_val) = args.steps {
                let steps: Vec<Step> = serde_json::from_value(steps_val)
                    .map_err(|e| AdkError::tool(format!("Invalid steps format: {}", e)))?;
                task.steps = steps;

                // Auto-mark status as Done if all steps are completed successfully
                if !task.steps.is_empty() && task.steps.iter().all(|s| s.completed) {
                    task.status = TaskStatus::Done;
                }
            }
            task.updated_at = Utc::now();

            save_states(&states).await?;
            Ok(json!({"status": "success", "message": format!("Task '{}' updated", args.task_id)}))
        } else {
            Err(AdkError::tool(format!("Task '{}' not found", args.task_id)))
        }
    }
}

pub struct GetTask;
#[async_trait::async_trait]
impl Tool for GetTask {
    fn name(&self) -> &str {
        "get_task"
    }
    fn description(&self) -> &str {
        "Retrieves the current state of a specific task."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The ID of the task." }
            },
            "required": ["task_id"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: TaskIdArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;
        let states = load_states().await?;
        if let Some(task) = states.into_iter().find(|t| t.task_id == args.task_id) {
            Ok(json!(task))
        } else {
            // Check archive as fallback
            let archive_path = get_archive_file().await?;
            if archive_path.exists() {
                let content = fs::read_to_string(&archive_path)
                    .await
                    .map_err(|e| AdkError::tool(format!("Failed to read archive: {}", e)))?;
                let archived: Vec<TaskState> = serde_json::from_str(&content).unwrap_or_default();
                if let Some(task) = archived.into_iter().find(|t| t.task_id == args.task_id) {
                    return Ok(json!(task));
                }
            }
            Err(AdkError::tool(format!("Task '{}' not found", args.task_id)))
        }
    }
}

pub struct ListActiveTasks;
#[async_trait::async_trait]
impl Tool for ListActiveTasks {
    fn name(&self) -> &str {
        "list_active_tasks"
    }
    fn description(&self) -> &str {
        "Lists all tasks that are currently active."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({ "type": "object", "properties": {} }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        _args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let states = load_states().await?;
        let active: Vec<_> = states
            .into_iter()
            .filter(|t| t.status.is_active())
            .collect();

        if active.is_empty() {
            Ok(json!({"message": "No active tasks found."}))
        } else {
            Ok(json!({ "active_tasks": active }))
        }
    }
}

pub struct ListTasks;
#[async_trait::async_trait]
impl Tool for ListTasks {
    fn name(&self) -> &str {
        "list_tasks"
    }
    fn description(&self) -> &str {
        "Lists all tasks, including active and archived/terminal ones."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({ "type": "object", "properties": {} }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        _args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let mut all_tasks = load_states().await?;
        let archive_path = get_archive_file().await?;
        if archive_path.exists() {
            let content = fs::read_to_string(&archive_path)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to read archive: {}", e)))?;
            let mut archived: Vec<TaskState> = serde_json::from_str(&content).unwrap_or_default();
            all_tasks.append(&mut archived);
        }

        Ok(json!({ "tasks": all_tasks }))
    }
}

pub struct DeleteTask;
#[async_trait::async_trait]
impl Tool for DeleteTask {
    fn name(&self) -> &str {
        "delete_task"
    }
    fn description(&self) -> &str {
        "Deletes a task from the active state manager or the archive."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The ID of the task to delete." }
            },
            "required": ["task_id"]
        }))
    }
    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: TaskIdArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let mut deleted = false;

        // 1. Attempt to delete from active states
        let mut states = load_states().await?;
        let len_before = states.len();
        states.retain(|t| t.task_id != args.task_id);
        if states.len() < len_before {
            save_states(&states).await?;
            deleted = true;
        }

        // 2. Attempt to delete from archived states
        let archive_path = get_archive_file().await?;
        if archive_path.exists() {
            let content = fs::read_to_string(&archive_path)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to read archive: {}", e)))?;
            let mut archived: Vec<TaskState> = serde_json::from_str(&content).unwrap_or_default();
            let len_archive_before = archived.len();
            archived.retain(|t| t.task_id != args.task_id);
            if archived.len() < len_archive_before {
                let serialized = serde_json::to_string_pretty(&archived)
                    .map_err(|e| AdkError::tool(format!("Failed to serialize archive: {}", e)))?;
                fs::write(&archive_path, serialized)
                    .await
                    .map_err(|e| AdkError::tool(format!("Failed to write archive: {}", e)))?;
                deleted = true;
            }
        }

        if deleted {
            Ok(json!({
                "status": "success",
                "message": format!("Task '{}' deleted successfully.", args.task_id)
            }))
        } else {
            Err(AdkError::tool(format!("Task '{}' not found", args.task_id)))
        }
    }
}

pub fn state_manager_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(InitTask),
        Arc::new(UpdateTask),
        Arc::new(GetTask),
        Arc::new(ListActiveTasks),
        Arc::new(ListTasks),
        Arc::new(DeleteTask),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_status_parsing() {
        assert_eq!("backlog".parse::<TaskStatus>().unwrap(), TaskStatus::Backlog);
        assert_eq!("todo".parse::<TaskStatus>().unwrap(), TaskStatus::Todo);
        assert_eq!("in_progress".parse::<TaskStatus>().unwrap(), TaskStatus::InProgress);
        assert_eq!("in_review".parse::<TaskStatus>().unwrap(), TaskStatus::InReview);
        assert_eq!("blocked".parse::<TaskStatus>().unwrap(), TaskStatus::Blocked);
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert_eq!("completed".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert_eq!("cancelled".parse::<TaskStatus>().unwrap(), TaskStatus::Cancelled);
        assert_eq!("failed".parse::<TaskStatus>().unwrap(), TaskStatus::Cancelled);
        assert_eq!("cancel".parse::<TaskStatus>().unwrap(), TaskStatus::Cancelled);
    }

    #[test]
    fn test_status_logic() {
        assert!(TaskStatus::Todo.is_active());
        assert!(TaskStatus::InProgress.is_active());
        assert!(TaskStatus::InReview.is_active());
        assert!(TaskStatus::Blocked.is_active());
        assert!(!TaskStatus::Backlog.is_active());
        assert!(!TaskStatus::Done.is_active());
        assert!(!TaskStatus::Cancelled.is_active());

        assert!(TaskStatus::Done.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
    }

    #[test]
    fn test_status_serde_aliases() {
        // Test backward compatibility aliases
        let completed: TaskStatus = serde_json::from_value(json!("completed")).unwrap();
        assert_eq!(completed, TaskStatus::Done);

        let failed: TaskStatus = serde_json::from_value(json!("failed")).unwrap();
        assert_eq!(failed, TaskStatus::Cancelled);

        // Test normal serialization
        let json = serde_json::to_value(TaskStatus::Done).unwrap();
        assert_eq!(json, json!("done"));
    }

    #[test]
    fn test_step_verification_criteria() {
        let step = Step {
            description: "Test".to_string(),
            completed: false,
            verification_criteria: Some("Criteria".to_string()),
            verification_method: None,
            phase: StepPhase::Implementation,
        };
        let json = serde_json::to_value(&step).unwrap();
        assert_eq!(json["verification_criteria"], json!("Criteria"));

        let deserialized: Step = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.verification_criteria, Some("Criteria".to_string()));
    }

    #[test]
    fn test_step_verification_method() {
        let step = Step {
            description: "Run unit tests".to_string(),
            completed: false,
            verification_criteria: None,
            verification_method: Some(VerificationMethod::Command {
                command: "cargo test".to_string(),
                expected_output: Some("ok".to_string()),
                allow_failure: false,
            }),
            phase: StepPhase::Implementation,
        };
        let json = serde_json::to_value(&step).unwrap();
        assert_eq!(json["verification_method"]["type"], json!("command"));
        assert_eq!(json["verification_method"]["command"], json!("cargo test"));

        let deserialized: Step = serde_json::from_value(json).unwrap();
        if let Some(VerificationMethod::Command { command, expected_output, allow_failure }) = deserialized.verification_method {
            assert_eq!(command, "cargo test");
            assert_eq!(expected_output, Some("ok".to_string()));
            assert!(!allow_failure);
        } else {
            panic!("Expected Command variant");
        }
    }
}

