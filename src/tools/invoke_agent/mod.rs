use adk_rust::Tool;
use adk_rust::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

pub struct InvokeAgent {
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl InvokeAgent {
    pub fn new(specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { specialists }
    }
}

#[async_trait::async_trait]
impl Tool for InvokeAgent {
    fn name(&self) -> &str {
        "invoke_agent"
    }

    fn description(&self) -> &str {
        "Invokes a single specialist agent by name with a given prompt. \
        Use this when you need to delegate a focused task to a specific expert agent \
        (e.g., 'coder', 'researcher', 'writer', 'generalist', 'verifier', 'ralph'). \
        For running multiple tasks simultaneously, use `parallel_tasks` instead."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "specialist": {
                    "type": "string",
                    "description": "The name of the specialist agent to invoke. Available agents: 'generalist', 'coder', 'researcher', 'writer', 'verifier', 'ralph'."
                },
                "prompt": {
                    "type": "string",
                    "description": "The task description or question to send to the specialist agent."
                }
            },
            "required": ["specialist", "prompt"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let specialist_name = args["specialist"]
            .as_str()
            .ok_or_else(|| AdkError::tool("Missing required field: 'specialist'"))?
            .to_string();

        let prompt = args["prompt"]
            .as_str()
            .ok_or_else(|| AdkError::tool("Missing required field: 'prompt'"))?
            .to_string();

        match self.specialists.get(&specialist_name) {
            Some(tool) => {
                match tool.execute(ctx, json!({ "input": prompt })).await {
                    Ok(result) => Ok(json!({
                        "status": "success",
                        "specialist": specialist_name,
                        "output": result
                    })),
                    Err(e) => Ok(json!({
                        "status": "error",
                        "specialist": specialist_name,
                        "error": format!("{}", e)
                    })),
                }
            }
            None => {
                let available: Vec<&String> = self.specialists.keys().collect();
                Ok(json!({
                    "status": "error",
                    "error": format!(
                        "Specialist '{}' not found. Available specialists: {:?}",
                        specialist_name, available
                    )
                }))
            }
        }
    }
}

pub fn invoke_agent_tool(specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(InvokeAgent::new(specialists))]
}

// Corrected test module
#[cfg(test)]
mod tests {
    use super::*;
    use adk_rust::{Tool, ToolContext};
    use adk_tool::SimpleToolContext;
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::json;

    struct MockSpecialist;

    #[async_trait::async_trait]
    impl Tool for MockSpecialist {
        fn name(&self) -> &str { "mock_specialist" }
        fn description(&self) -> &str { "mock specialist" }
        async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: serde_json::Value) -> std::result::Result<serde_json::Value, adk_rust::AdkError> {
            Ok(json!({ "output": format!("Handled: {}", args["input"]) }))
        }
    }

    #[tokio::test]
    async fn test_missing_fields() {
        let specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        let tool = InvokeAgent::new(specialists);
        let ctx = Arc::new(SimpleToolContext::new("test_caller"));
        // Missing prompt
        let args_missing_prompt = json!({ "specialist": "coder" });
        let result = tool.execute(ctx.clone(), args_missing_prompt).await;
        assert!(result.is_err());
        // Missing specialist
        let args_missing_specialist = json!({ "prompt": "Hello" });
        let result = tool.execute(ctx, args_missing_specialist).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nonexistent_specialist() {
        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("mock".to_string(), Arc::new(MockSpecialist));
        let tool = InvokeAgent::new(specialists);
        let ctx = Arc::new(SimpleToolContext::new("test_caller"));
        let args = json!({ "specialist": "nonexistent", "prompt": "Do something" });
        let result = tool.execute(ctx, args).await.unwrap();
        assert_eq!(result["status"], "error");
        assert!(result["error"].as_str().unwrap().contains("not found"));
    }
}
