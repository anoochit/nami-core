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
    /// The name of the specialized agent to handle this task (e.g., 'generalist', 'coder', 'researcher', 'writer', 'ralph').
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
        let mut available: Vec<String> = self.specialists.keys().cloned().collect();
        available.sort();
        let available_str = available.join(", ");
        let desc = format!(
            "The name of the sub-agent to use. Available: {}.",
            available_str
        );
        Some(json!({
            "type": "object",
            "properties": {
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string", "description": "The prompt or instructions for the sub-agent." },
                            "specialist": { 
                                "type": "string", 
                                "description": desc
                            }
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

        let mut synthesis_summary = String::new();
        synthesis_summary.push_str("=== Parallel Tasks Execution Summary & Aggregated State ===\n");
        for (i, res) in final_results.iter().enumerate() {
            synthesis_summary.push_str(&format!("Task {}:\n{}\n\n", i + 1, res));
        }

        Ok(json!({
            "status": "success",
            "tasks_executed": final_results.len(),
            "outputs": final_results,
            "aggregated_state": synthesis_summary
        }))
    }
}

pub fn parallel_tasks_tool(specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ParallelTasks::new(specialists))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_rust::{Tool, ToolContext};
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockTool;
    #[async_trait::async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str { "mock" }
        fn description(&self) -> &str { "mock" }
        async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
            Ok(json!({ "output": format!("Processed: {}", args["input"]) }))
        }
    }

    #[tokio::test]
    async fn test_parallel_tasks() {
        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("coder".to_string(), Arc::new(MockTool) as Arc<dyn Tool>);
        specialists.insert("writer".to_string(), Arc::new(MockTool) as Arc<dyn Tool>);

        let parallel_tool = ParallelTasks::new(specialists);
        
        // Use a simple tool context for testing
        let ctx = Arc::new(adk_tool::SimpleToolContext::new("test_caller"));

        let args = json!({
            "tasks": [
                { "prompt": "Write code", "specialist": "coder" },
                { "prompt": "Write docs", "specialist": "writer" }
            ]
        });

        let result = parallel_tool.execute(ctx, args).await.unwrap();
        
        assert_eq!(result["status"], "success");
        assert_eq!(result["tasks_executed"], 2);
        
        let outputs = result["outputs"].as_array().unwrap();
        assert!(outputs[0].as_str().unwrap().contains("success"));
        assert!(outputs[1].as_str().unwrap().contains("success"));
    }
}
