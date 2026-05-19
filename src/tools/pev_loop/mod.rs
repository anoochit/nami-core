use adk_rust::Tool;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use futures::StreamExt;
use crate::tools::state_manager::{InitTask, UpdateTask, GetTask};

#[derive(Deserialize, JsonSchema)]
pub struct PevInitArgs {
    /// The high-level objective to achieve.
    pub goal: String,
    /// Unique identifier for the task.
    pub task_id: String,
}

pub struct PevInit {
    model: Arc<dyn Llm>,
}

impl PevInit {
    pub fn new(model: Arc<dyn Llm>) -> Self {
        Self { model }
    }
}

#[async_trait::async_trait]
impl Tool for PevInit {
    fn name(&self) -> &str {
        "pev_init"
    }

    fn description(&self) -> &str {
        "Initializes a PEV (Planner-Executor-Verifier) task by generating a plan with explicit verification criteria."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string", "description": "The high-level objective to achieve." },
                "task_id": { "type": "string", "description": "Unique identifier for the task." }
            },
            "required": ["goal", "task_id"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: PevInitArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let prompt = format!(
            r#"You are a high-level Planner. Break down the following goal into a sequence of executable steps.
            For EACH step, you MUST provide:
            1. description: What needs to be done.
            2. verification_criteria: Clear, objective criteria to judge if the step is finished correctly.

            Goal: {}

            Return a JSON array of objects: [{{"description": "...", "verification_criteria": "..."}}]"#,
            args.goal
        );

        let mut stream = self.model.generate_content(
            LlmRequest::new("planner", vec![Content::new("user").with_text(prompt)]),
            false,
        ).await.map_err(|e| AdkError::tool(format!("Planner failed: {}", e)))?;

        let mut text = String::new();
        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| AdkError::tool(format!("Stream error: {}", e)))?;
            if let Some(content) = event.content {
                for part in content.parts {
                    if let Some(t) = part.text() {
                        text.push_str(t);
                    }
                }
            }
        }

        // Clean up markdown code blocks if present
        let cleaned_text = text.trim().trim_start_matches("```json").trim_end_matches("```").trim();
        let steps_val: Value = serde_json::from_str(cleaned_text)
            .map_err(|e| AdkError::tool(format!("Failed to parse planner output as JSON: {}. Output was: {}", e, cleaned_text)))?;

        let init_tool = InitTask {};
        init_tool.execute(ctx, json!({
            "task_id": args.task_id,
            "goal": args.goal,
            "steps": steps_val
        })).await?;

        Ok(json!({
            "status": "success",
            "message": format!("PEV Task '{}' initialized with planned steps.", args.task_id)
        }))
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct PevRunArgs {
    /// The ID of the PEV task to run.
    pub task_id: String,
    /// The specialist to use for execution (default: coder).
    pub executor: Option<String>,
    /// Maximum number of iterations to run.
    pub max_iterations: Option<usize>,
}

pub struct PevRun {
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl PevRun {
    pub fn new(specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { specialists }
    }
}

#[async_trait::async_trait]
impl Tool for PevRun {
    fn name(&self) -> &str {
        "pev_run"
    }

    fn description(&self) -> &str {
        "Runs the PEV loop for a specific task. Iteratively executes steps and verifies them using a critic."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The ID of the PEV task to run." },
                "executor": { "type": "string", "description": "The specialist to use for execution (e.g., coder, researcher)." },
                "max_iterations": { "type": "integer", "description": "Maximum number of steps to execute in this run." }
            },
            "required": ["task_id"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: PevRunArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let executor_name = args.executor.unwrap_or_else(|| "coder".to_string());
        let executor = self.specialists.get(&executor_name)
            .ok_or_else(|| AdkError::tool(format!("Executor specialist '{}' not found", executor_name)))?;
        
        let verifier = self.specialists.get("verifier")
            .ok_or_else(|| AdkError::tool("Verifier specialist not found"))?;

        let mut iterations = 0;
        let max_iters = args.max_iterations.unwrap_or(5);
        let mut log_entries = Vec::new();

        loop {
            if iterations >= max_iters {
                break;
            }
            iterations += 1;

            // 1. Get current task state
            let get_tool = GetTask {};
            let task_val = get_tool.execute(ctx.clone(), json!({"task_id": args.task_id})).await?;
            let mut task: crate::tools::state_manager::TaskState = serde_json::from_value(task_val)
                .map_err(|e| AdkError::tool(format!("Failed to parse task state: {}", e)))?;

            if task.status.is_terminal() {
                break;
            }

            // 2. Find next incomplete step
            let step_index = task.steps.iter().position(|s| !s.completed);
            let Some(idx) = step_index else {
                // All steps completed
                let update_tool = UpdateTask {};
                update_tool.execute(ctx.clone(), json!({
                    "task_id": args.task_id,
                    "status": "done"
                })).await?;
                break;
            };

            let (step_desc, step_criteria) = {
                let step = &task.steps[idx];
                (step.description.clone(), step.verification_criteria.clone())
            };
            log::info!("PEV Iteration {}: Executing step: {}", iterations, step_desc);

            // 3. EXECUTE
            let exec_prompt = format!(
                "GOAL: {}\nSTEP: {}\nLAST FEEDBACK: {}\n\nPlease complete this step.",
                task.goal,
                step_desc,
                task.last_step.as_deref().unwrap_or("None")
            );

            let exec_res = executor.execute(ctx.clone(), json!({"input": exec_prompt})).await?;
            let exec_output = exec_res.to_string();

            // 4. VERIFY
            let verify_prompt = format!(
                "OBJECTIVE: {}\nSTEP: {}\nCRITERIA: {}\nOUTPUT TO VERIFY: {}\n\nDoes this meet the criteria? If yes, say 'VERIFIED'. If no, list the problems.",
                task.goal,
                step_desc,
                step_criteria.as_deref().unwrap_or("Use your best judgment."),
                exec_output
            );

            let verify_res = verifier.execute(ctx.clone(), json!({"input": verify_prompt})).await?;
            let verify_output = verify_res.to_string();

            // 5. UPDATE STATE
            let update_tool = UpdateTask {};
            if verify_output.contains("VERIFIED") {
                log::info!("PEV Iteration {}: Step VERIFIED", iterations);
                task.steps[idx].completed = true;
                update_tool.execute(ctx.clone(), json!({
                    "task_id": args.task_id,
                    "steps": task.steps,
                    "last_step": format!("Step '{}' verified successfully.", step_desc)
                })).await?;
                log_entries.push(format!("Step {}: VERIFIED", idx + 1));
            } else {
                log::info!("PEV Iteration {}: Step FAILED verification", iterations);
                update_tool.execute(ctx.clone(), json!({
                    "task_id": args.task_id,
                    "last_step": format!("Verification failed for '{}': {}", step_desc, verify_output)
                })).await?;
                log_entries.push(format!("Step {}: FAILED - {}", idx + 1, verify_output));
            }
        }

        Ok(json!({
            "status": "completed",
            "iterations": iterations,
            "log": log_entries
        }))
    }
}

pub fn pev_tools(model: Arc<dyn Llm>, specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(PevInit::new(model)),
        Arc::new(PevRun::new(specialists)),
    ]
}
