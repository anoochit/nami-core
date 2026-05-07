use adk_rust::Tool;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use futures::future::join_all;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct Task {
    /// The specific task or prompt for this job.
    pub prompt: String,
    /// The name of the specialized agent to handle this task (e.g., 'generalist').
    pub specialist: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ParallelTasksArgs {
    /// A list of tasks to execute simultaneously.
    pub tasks: Vec<Task>,
}

pub struct ParallelTasks {
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl ParallelTasks {
    pub fn new(specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { specialists }
    }
}

#[async_trait::async_trait]
impl Tool for ParallelTasks {
    fn name(&self) -> &str {
        "parallel_tasks"
    }

    fn description(&self) -> &str {
        "Executes multiple tasks in parallel using sub-agents. Use this for high-speed multi-tasking."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string", "description": "The prompt or instructions for the sub-agent." },
                            "specialist": { "type": "string", "description": "The name of the sub-agent to use (e.g., 'generalist')." }
                        },
                        "required": ["prompt", "specialist"]
                    }
                }
            },
            "required": ["tasks"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: ParallelTasksArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let mut futures = Vec::new();

        for task in args.tasks {
            if let Some(tool) = self.specialists.get(&task.specialist) {
                let tool = tool.clone();
                let prompt = task.prompt.clone();
                let ctx = ctx.clone();
                let specialist_name = task.specialist.clone();

                futures.push(tokio::spawn(async move {
                    match tool.execute(ctx, json!({ "input": prompt })).await {
                        Ok(res) => format!("[{}] success: {}", specialist_name, res),
                        Err(e) => format!("[{}] error: {}", specialist_name, e),
                    }
                }));
            } else {
                let specialist = task.specialist.clone();
                futures.push(tokio::spawn(async move {
                    format!("Error: Specialist '{}' not found", specialist)
                }));
            }
        }

        let results = join_all(futures).await;
        let mut final_results = Vec::new();

        for res in results {
            match res {
                Ok(r) => final_results.push(r),
                Err(e) => final_results.push(format!("Internal error: {}", e)),
            }
        }

        Ok(json!({
            "status": "success",
            "tasks_executed": final_results.len(),
            "outputs": final_results
        }))
    }
}

pub fn parallel_tasks_tool(specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ParallelTasks::new(specialists))]
}
