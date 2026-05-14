use crate::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_rust::tool::ToolContext;
use adk_tool::AdkError;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::fs;
use crate::tools::state_manager::InitTask;

#[derive(Deserialize, JsonSchema)]
struct PlanCreateArgs {
    name: String,
    objective: String,
}

pub struct PlanCreate;
#[async_trait::async_trait]
impl Tool for PlanCreate {
    fn name(&self) -> &str {
        "plan_create"
    }
    fn description(&self) -> &str {
        "Creates a new implementation plan, initializes it as a task, and writes the plan document."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the plan." },
                "objective": { "type": "string", "description": "The goal/objective of the task." }
            },
            "required": ["name", "objective"]
        }))
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: PlanCreateArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        
        // 1. Define initial steps based on objective
        let steps = vec![
            format!("Analyze current implementation for: {}", args.objective),
            format!("Design solution for: {}", args.objective),
            "Implement core logic".to_string(),
            "Add error handling and logging".to_string(),
            "Verify with unit and integration tests".to_string(),
        ];
        
        // 2. Initialize task using state_manager::InitTask
        let init_args = json!({
            "task_id": args.name.replace(" ", "-"),
            "goal": args.objective,
            "steps": steps
        });
        let init_tool = InitTask {};
        init_tool.execute(ctx.clone(), init_args).await?;
        
        // 3. Generate plan markdown document from task state
        let root = get_workspace_dir().await?;
        let plans_dir = root.join("plans");
        fs::create_dir_all(&plans_dir).await.map_err(|e| AdkError::tool(e.to_string()))?;
        
        let normalized_name = args.name.replace(" ", "-");
        let path = plans_dir.join(format!("{}.md", normalized_name));
        
        let mut content = format!("# Plan: {}\n\n## Objective\n{}\n\n## Implementation Steps\n", args.name, args.objective);
        for (i, step) in steps.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, step));
        }
        
        content.push_str("\n---\n*This plan is synced with an active task in the state manager.*");
        
        fs::write(&path, content).await.map_err(|e| AdkError::tool(e.to_string()))?;
        
        Ok(json!({
            "status": "success", 
            "path": format!("plans/{}.md", normalized_name),
            "task_initialized": true
        }))
    }
}

#[derive(Deserialize, JsonSchema)]
struct PlanNameArgs {
    name: String,
}

pub struct PlanShow;
#[async_trait::async_trait]
impl Tool for PlanShow {
    fn name(&self) -> &str {
        "plan_show"
    }
    fn description(&self) -> &str {
        "Reads and displays an implementation plan from workspace/plans/."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the plan." }
            },
            "required": ["name"]
        }))
    }
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: PlanNameArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        let root = get_workspace_dir().await?;
        let path = root.join("plans").join(format!("{}.md", args.name));
        
        let content = fs::read_to_string(&path).await.map_err(|e| AdkError::tool(e.to_string()))?;
        Ok(json!({"content": content}))
    }
}

pub struct PlanList;
#[async_trait::async_trait]
impl Tool for PlanList {
    fn name(&self) -> &str {
        "plan_list"
    }
    fn description(&self) -> &str {
        "Lists all available implementation plans."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({"type": "object", "properties": {}}))
    }
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> std::result::Result<Value, AdkError> {
        let root = get_workspace_dir().await?;
        let plans_dir = root.join("plans");
        
        let mut entries = Vec::new();
        if let Ok(mut dir) = fs::read_dir(plans_dir).await {
            while let Ok(Some(entry)) = dir.next_entry().await {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                    entries.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        
        Ok(json!({"plans": entries}))
    }
}

pub fn plan_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(PlanCreate),
        Arc::new(PlanShow),
        Arc::new(PlanList),
    ]
}
