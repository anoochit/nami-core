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
