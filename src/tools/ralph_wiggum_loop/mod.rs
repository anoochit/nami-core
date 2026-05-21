use adk_rust::Tool;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct RalphWiggumLoopArgs {
    /// The goal to achieve.
    pub goal: String,
    /// The condition that stops the loop.
    pub stop_condition: String,
}

pub struct RalphWiggumLoop {
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl RalphWiggumLoop {
    pub fn new(specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { specialists }
    }
}

#[async_trait::async_trait]
impl Tool for RalphWiggumLoop {
    fn name(&self) -> &str {
        "ralph_wiggum_loop"
    }

    fn description(&self) -> &str {
        "Runs an autonomous loop with Ralph Wiggum to achieve a goal. It continues until the stop condition is met."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string", "description": "The goal you want to achieve." },
                "stop_condition": { "type": "string", "description": "When should Ralph stop trying?" }
            },
            "required": ["goal", "stop_condition"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: RalphWiggumLoopArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let ralph = self
            .specialists
            .get("ralph")
            .ok_or_else(|| AdkError::tool("Specialist 'ralph' not found"))?;

        let mut current_state = format!(
            "Goal: {}\nStop Condition: {}",
            args.goal, args.stop_condition
        );
        let mut outputs = Vec::new();
        let max_iterations = 5;

        for i in 1..=max_iterations {
            let prompt = format!(
                "Iteration {}: \nCurrent Progress: {}\nKeep working toward the goal. If you are finished, say 'I'm a winner!'",
                i, current_state
            );

            match ralph.execute(ctx.clone(), json!({ "input": prompt })).await {
                Ok(res) => {
                    let output_str = res.to_string();
                    outputs.push(format!("Iteration {}: {}", i, output_str));

                    if output_str.contains("I'm a winner!") {
                        break;
                    }

                    current_state = format!("{}\nLast Action: {}", current_state, output_str);
                }
                Err(e) => {
                    outputs.push(format!("Iteration {} Error: {}", i, e));
                    break;
                }
            }
        }

        Ok(json!({
            "status": "completed",
            "iterations": outputs.len(),
            "log": outputs
        }))
    }
}

pub fn ralph_wiggum_loop_tool(specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(RalphWiggumLoop::new(specialists))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Mutex;

    struct MockRalph {
        responses: Vec<String>,
        current_response: Mutex<usize>,
    }

    #[async_trait]
    impl Tool for MockRalph {
        fn name(&self) -> &str { "ralph" }
        fn description(&self) -> &str { "mock ralph" }
        async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> std::result::Result<Value, AdkError> {
            let mut idx = self.current_response.lock().unwrap();
            let response = self.responses.get(*idx).cloned().unwrap_or_else(|| "I'm a winner!".to_string());
            *idx += 1;
            Ok(json!(response))
        }
    }

    struct MockContext;
    impl adk_rust::ReadonlyContext for MockContext {
        fn invocation_id(&self) -> &str { "" }
        fn agent_name(&self) -> &str { "" }
        fn user_id(&self) -> &str { "" }
        fn app_name(&self) -> &str { "" }
        fn session_id(&self) -> &str { "" }
        fn branch(&self) -> &str { "" }
        fn user_content(&self) -> &adk_rust::Content { todo!() }
    }
    impl adk_rust::CallbackContext for MockContext {
        fn artifacts(&self) -> Option<Arc<dyn adk_rust::Artifacts>> { None }
    }
    #[async_trait]
    impl adk_rust::tool::ToolContext for MockContext {
        fn function_call_id(&self) -> &str { "" }
        fn actions(&self) -> adk_core::event::EventActions { todo!() }
        fn set_actions(&self, _: adk_core::event::EventActions) { todo!() }
        async fn search_memory(&self, _: &str) -> std::result::Result<Vec<adk_rust::MemoryEntry>, adk_rust::AdkError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_ralph_wiggum_loop_success() {
        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("ralph".to_string(), Arc::new(MockRalph {
            responses: vec![
                "I'm gluing my head to the shoulder!".to_string(),
                "I'm a winner!".to_string(),
            ],
            current_response: Mutex::new(0),
        }));

        let tool = RalphWiggumLoop::new(specialists);
        let ctx = Arc::new(MockContext);
        let args = json!({
            "goal": "Win a gold medal",
            "stop_condition": "When I say I'm a winner"
        });

        let result = tool.execute(ctx, args).await.unwrap();
        
        assert_eq!(result["status"], "completed");
        assert_eq!(result["iterations"], 2);
        let log = result["log"].as_array().unwrap();
        assert!(log[0].as_str().unwrap().contains("I'm gluing my head to the shoulder!"));
        assert!(log[1].as_str().unwrap().contains("I'm a winner!"));
    }

    #[tokio::test]
    async fn test_ralph_wiggum_loop_max_iterations() {
        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("ralph".to_string(), Arc::new(MockRalph {
            responses: vec!["Still trying".to_string(); 5],
            current_response: Mutex::new(0),
        }));

        let tool = RalphWiggumLoop::new(specialists);
        let ctx = Arc::new(MockContext);
        let args = json!({
            "goal": "Infinity",
            "stop_condition": "Never"
        });

        let result = tool.execute(ctx, args).await.unwrap();
        
        assert_eq!(result["status"], "completed");
        assert_eq!(result["iterations"], 5);
    }

    #[tokio::test]
    async fn test_ralph_wiggum_loop_specialist_not_found() {
        let specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        let tool = RalphWiggumLoop::new(specialists);
        let ctx = Arc::new(MockContext);
        let args = json!({
            "goal": "Fail",
            "stop_condition": "Now"
        });

        let result = tool.execute(ctx, args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Specialist 'ralph' not found"));
    }

    #[tokio::test]
    async fn test_ralph_wiggum_loop_specialist_error() {
        struct ErrorTool;
        #[async_trait]
        impl Tool for ErrorTool {
            fn name(&self) -> &str { "ralph" }
            fn description(&self) -> &str { "error" }
            async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> std::result::Result<Value, AdkError> {
                Err(AdkError::tool("Execution failed"))
            }
        }

        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("ralph".to_string(), Arc::new(ErrorTool));

        let tool = RalphWiggumLoop::new(specialists);
        let ctx = Arc::new(MockContext);
        let args = json!({
            "goal": "Try and fail",
            "stop_condition": "Now"
        });

        let result = tool.execute(ctx, args).await.unwrap();
        assert_eq!(result["status"], "completed");
        assert_eq!(result["iterations"], 1);
        let log = result["log"].as_array().unwrap();
        let log_entry = log[0].as_str().unwrap();
        assert!(log_entry.contains("Iteration 1 Error:"));
        assert!(log_entry.contains("Execution failed"));
    }
}
