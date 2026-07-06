use crate::utils::{get_workspace_dir, get_nami_dir};
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_rust::tool::ToolContext;
use crossterm::style::Stylize;
use adk_rust::prelude::*;
use adk_tool::AdkError;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use futures::StreamExt;
use crate::tools::state_manager::{InitTask, UpdateTask, GetTask, VerificationMethod};

/// Utility function to clean markdown fences and extract a JSON substring if necessary.
fn clean_json_string(s: &str) -> String {
    let mut s = s.trim().to_string();
    
    // Strip markdown JSON code blocks if present
    if s.starts_with("```") {
        if let Some(end_idx) = s.rfind("```") {
            if end_idx > 3 {
                let start_idx = if s.starts_with("```json") { 7 } else { 3 };
                s = s[start_idx..end_idx].trim().to_string();
            }
        }
    }
    
    // Find first brace/bracket and last brace/bracket to extract raw JSON
    let first_brace = s.find('{');
    let first_bracket = s.find('[');
    let last_brace = s.rfind('}');
    let last_bracket = s.rfind(']');
    
    match (first_brace, first_bracket, last_brace, last_bracket) {
        (Some(fb), Some(fk), Some(lb), Some(lk)) => {
            if fb < fk && lb > lk {
                s = s[fb..=lb].to_string();
            } else if fk < fb && lk > lb {
                s = s[fk..=lk].to_string();
            } else if fb < fk {
                s = s[fb..=lb].to_string();
            } else {
                s = s[fk..=lk].to_string();
            }
        }
        (Some(fb), None, Some(lb), None) => {
            s = s[fb..=lb].to_string();
        }
        (None, Some(fk), None, Some(lk)) => {
            s = s[fk..=lk].to_string();
        }
        _ => {}
    }
    
    // Clean trailing commas in objects and arrays to prevent common serde failures
    s = s.replace(",\n}", "\n}").replace(",\r\n}", "\r\n}").replace(",}", "}");
    s = s.replace(",\n]", "\n]").replace(",\r\n]", "\r\n]").replace(",]", "]");

    s
}

/// Helper to parse JSON with self-healing fallback when model output is slightly malformed.
async fn parse_json_with_healing(
    model: &Arc<dyn Llm>,
    initial_text: &str,
    system_role: &str,
    user_prompt: &str,
) -> std::result::Result<Value, AdkError> {
    let cleaned_initial = clean_json_string(initial_text);
    if let Ok(val) = serde_json::from_str::<Value>(&cleaned_initial) {
        return Ok(val);
    }
    
    log::info!("JSON parsing failed. Attempting healing session with the LLM...");
    let healing_prompt = format!(
        "The following text was expected to be a valid JSON object/array matching instructions, but failed to parse as valid JSON:\n\n\
        === INVALID TEXT ===\n\
        {}\n\
        ====================\n\n\
        Please rewrite it to be 100% valid JSON. Do NOT include any explanation or conversational text, only the JSON block.\n\
        Original instruction/intent:\n\
        {}",
        initial_text,
        user_prompt
    );

    let mut stream = model.generate_content(
        LlmRequest::new(system_role, vec![Content::new("user").with_text(healing_prompt)]),
        false,
    ).await.map_err(|e| AdkError::tool(format!("Llm healing request failed: {}", e)))?;

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

    let cleaned_healed = clean_json_string(&text);
    serde_json::from_str::<Value>(&cleaned_healed)
        .map_err(|e| AdkError::tool(format!(
            "Failed to parse healed JSON: {}.\nOriginal text: {}\nHealed text: {}",
            e, initial_text, text
        )))
}

#[derive(Deserialize, JsonSchema)]
struct PlanCreateArgs {
    name: String,
    objective: String,
    autonomous: Option<bool>,
    steps: Option<Vec<Value>>,
}

pub struct PlanCreate {
    model: Option<Arc<dyn Llm>>,
}

impl PlanCreate {
    pub fn new(model: Arc<dyn Llm>) -> Self {
        Self { model: Some(model) }
    }
}

#[async_trait::async_trait]
impl Tool for PlanCreate {
    fn name(&self) -> &str {
        "plan_create"
    }
    fn description(&self) -> &str {
        "Creates a new implementation plan, initializes it as a task, and writes the plan document. Supports autonomous planning."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the plan." },
                "objective": { "type": "string", "description": "The goal/objective of the task." },
                "autonomous": { "type": "boolean", "description": "Whether to use LLM to dynamically generate precise task steps with verification criteria." },
                "steps": { "type": "array", "items": { "type": "object" }, "description": "Pre-synthesized plan steps." }
            },
            "required": ["name", "objective"]
        }))
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: PlanCreateArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        
        let mut steps = args.steps.unwrap_or_default();
        let autonomous_mode = args.autonomous.unwrap_or(false);

        // 1. Generate steps
        if autonomous_mode && self.model.is_some() {
            let model = self.model.as_ref().unwrap();
            let prompt = format!(
                r#"You are a high-level Planner. Break down the following goal into a sequence of executable steps.
                You MUST structure the steps in two distinct phases:
                1. Implementation Phase: Group all code-writing, creation, and modification steps first.
                2. Verification Phase: Place a final, explicit testing and verification step at the very end to validate the entire implementation after it is complete.

                For EACH step, you MUST provide:
                1. description: What needs to be done.
                2. verification_criteria: Clear, objective criteria to judge if the step is finished correctly.

                Goal: {}

                Return a JSON array of objects: [{{"description": "...", "verification_criteria": "..."}}]"#,
                args.objective
            );

            if let Ok(mut stream) = model.generate_content(
                LlmRequest::new("planner", vec![Content::new("user").with_text(prompt.clone())]),
                false,
            ).await {
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

                if let Ok(steps_val) = parse_json_with_healing(model, &text, "planner", &prompt).await {
                    if let Some(arr) = steps_val.as_array() {
                        for item in arr {
                            steps.push(json!({
                                "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                                "verification_criteria": item.get("verification_criteria").and_then(|v| v.as_str()).unwrap_or("Use your best judgment.")
                            }));
                        }
                    }
                }
            }
        }

        // Fallback to architect/default steps if not autonomous or if autonomous generation failed
        if steps.is_empty() {
            if let Some(ref model) = self.model {
                let prompt = format!(
                    "You are an expert software architect. Create a detailed, sequential, step-by-step implementation plan for the following objective:\n\n\
                    Objective: {}\n\n\
                    Generate a JSON array of strings, where each string represents a clear, actionable step. E.g., [\"Analyze current...\", \"Create file X...\"].\n\
                    Do not include any markdown fences or explanatory text, just the raw JSON array.",
                    args.objective
                );

                if let Ok(mut stream) = model.generate_content(
                    LlmRequest::new("architect", vec![Content::new("user").with_text(prompt)]),
                    false,
                ).await {
                    let mut text = String::new();
                    while let Some(Ok(event)) = stream.next().await {
                        if let Some(content) = event.content {
                            for part in content.parts {
                                if let Some(t) = part.text() {
                                    text.push_str(t);
                                }
                            }
                        }
                    }
                    
                    let mut cleaned = text.trim().to_string();
                    if cleaned.starts_with("```") {
                        if let Some(end_idx) = cleaned.rfind("```") {
                            if end_idx > 3 {
                                let start_idx = if cleaned.starts_with("```json") { 7 } else { 3 };
                                cleaned = cleaned[start_idx..end_idx].trim().to_string();
                            }
                        }
                    }
                    
                    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&cleaned) {
                        for val in arr {
                            if let Some(s) = val.as_str() {
                                steps.push(json!({ "description": s }));
                            }
                        }
                    }
                }
            }
        }

        // Absolute fallback to generic templates
        if steps.is_empty() {
            steps = vec![
                json!({ "description": format!("Analyze current implementation for: {}", args.objective) }),
                json!({ "description": format!("Design solution for: {}", args.objective) }),
                json!({ "description": "Implement core logic" }),
                json!({ "description": "Add error handling and logging" }),
                json!({ "description": "Verify and tests" }),
            ];
        }
        
        // 2. Initialize task using state_manager::InitTask
        let normalized_name = args.name.replace(" ", "-");
        let init_args = json!({
            "task_id": normalized_name,
            "goal": args.objective,
            "steps": steps
        });
        let init_tool = InitTask {};
        init_tool.execute(ctx.clone(), init_args).await?;
        
        let plans_dir = get_nami_dir().join("plans");
        fs::create_dir_all(&plans_dir).await.map_err(|e| AdkError::tool(e.to_string()))?;
        
        let path = plans_dir.join(format!("{}.md", normalized_name));
        
        let mut content = format!("# Plan: {}\n\n## Objective\n{}\n\n## Implementation Steps\n", args.name, args.objective);
        for (i, step) in steps.iter().enumerate() {
            let desc = step.get("description").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(criteria) = step.get("verification_criteria").and_then(|v| v.as_str()) {
                content.push_str(&format!("{}. {}\n   - *Verification Criteria*: {}\n", i + 1, desc, criteria));
            } else {
                content.push_str(&format!("{}. {}\n", i + 1, desc));
            }
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
        let path = get_nami_dir().join("plans").join(format!("{}.md", args.name.replace(" ", "-")));
        
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
        let plans_dir = get_nami_dir().join("plans");
        
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

pub struct PlanDelete;
#[async_trait::async_trait]
impl Tool for PlanDelete {
    fn name(&self) -> &str {
        "plan_delete"
    }
    fn description(&self) -> &str {
        "Deletes an implementation plan from workspace/plans/."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the plan to delete." }
            },
            "required": ["name"]
        }))
    }
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: PlanNameArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        let path = get_nami_dir().join("plans").join(format!("{}.md", args.name.replace(" ", "-")));
        
        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| AdkError::tool(e.to_string()))?;
            Ok(json!({"status": "success", "message": format!("Plan '{}' deleted.", args.name)}))
        } else {
            Err(AdkError::tool(format!("Plan '{}' not found.", args.name)))
        }
    }
}

#[derive(Deserialize, JsonSchema)]
struct PlanUpdateArgs {
    name: String,
    steps: Vec<String>,
}

pub struct PlanUpdate;
#[async_trait::async_trait]
impl Tool for PlanUpdate {
    fn name(&self) -> &str {
        "plan_update"
    }
    fn description(&self) -> &str {
        "Updates an existing implementation plan, updates the associated task state, and rewrites the plan document."
    }
    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the plan to update." },
                "steps": { "type": "array", "items": { "type": "string" }, "description": "New sequence of steps." }
            },
            "required": ["name", "steps"]
        }))
    }
    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: PlanUpdateArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        let normalized_name = args.name.replace(" ", "-");
        let path = get_nami_dir().join("plans").join(format!("{}.md", normalized_name));
        
        if !path.exists() {
            return Err(AdkError::tool(format!("Plan '{}' not found.", args.name)));
        }

        // 1. Update task state
        let update_args = json!({
            "task_id": normalized_name,
            "steps": args.steps.iter().map(|s| json!({ "description": s })).collect::<Vec<_>>()
        });
        let update_tool = crate::tools::state_manager::UpdateTask {};
        update_tool.execute(ctx, update_args).await?;
        
        // 2. Rewrite plan markdown
        let mut content = format!("# Plan: {}\n\n## Implementation Steps\n", args.name);
        for (i, step) in args.steps.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, step));
        }
        
        content.push_str("\n---\n*This plan is synced with an active task in the state manager.*");
        
        fs::write(&path, content).await.map_err(|e| AdkError::tool(e.to_string()))?;
        
        Ok(json!({
            "status": "success", 
            "message": format!("Plan '{}' updated.", args.name)
        }))
    }
}

async fn run_llm_verification(
    _model: &Arc<dyn Llm>,
    verifier: &Arc<dyn Tool>,
    ctx: Arc<dyn ToolContext>,
    goal: &str,
    step_desc: &str,
    step_criteria: Option<&str>,
    exec_output: &str,
) -> (bool, String) {
    let verify_prompt = format!(
        "OBJECTIVE: {}\nSTEP: {}\nCRITERIA: {}\nOUTPUT TO VERIFY: {}\n\n\
        Analyze the output against the criteria. You MUST respond with a JSON object of this structure:\n\
        {{\n\
          \"verified\": true or false,\n\
          \"reasoning\": \"Your detailed reasoning here\",\n\
          \"suggested_fixes\": \"Suggestions for the executor if not verified\"\n\
        }}\n\
        Ensure the JSON is well-formed.",
        goal,
        step_desc,
        step_criteria.unwrap_or("Use your best judgment."),
        exec_output
    );

    match verifier.execute(ctx, json!({"input": verify_prompt})).await {
        Ok(verify_res) => {
            let verify_output = verify_res.to_string();
            let clean_verify = clean_json_string(&verify_output);
            if let Ok(verify_json) = serde_json::from_str::<Value>(&clean_verify) {
                let verified = verify_json.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
                let reasoning = verify_json.get("reasoning").and_then(|r| r.as_str()).unwrap_or("");
                let suggested_fixes = verify_json.get("suggested_fixes").and_then(|sf| sf.as_str()).unwrap_or("");
                
                let reason = if verified {
                    reasoning.to_string()
                } else {
                    format!("Reasoning: {}\nSuggested Fixes: {}", reasoning, suggested_fixes)
                };
                (verified, reason)
            } else {
                log::warn!("Failed to parse verifier output as JSON. Falling back to substring matching.");
                let lower_output = verify_output.to_lowercase();
                let verified = lower_output.contains("verified") || lower_output.contains("\"verified\": true") || lower_output.contains("true");
                (verified, verify_output)
            }
        }
        Err(e) => (false, format!("Verifier error: {}", e))
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct PlanExecuteArgs {
    /// The name of the plan to execute.
    pub name: String,
    /// The specialist to use for execution (default: coder).
    pub executor: Option<String>,
    /// Maximum number of iterations to run.
    pub max_iterations: Option<usize>,
}

pub struct PlanExecute {
    model: Arc<dyn Llm>,
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl PlanExecute {
    pub fn new(model: Arc<dyn Llm>, specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { model, specialists }
    }
}

#[async_trait::async_trait]
impl Tool for PlanExecute {
    fn name(&self) -> &str {
        "plan_execute"
    }

    fn description(&self) -> &str {
        "Executes a generated plan autonomously. Iteratively handles steps, tests outcomes, and self-corrects on verification failures."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The name of the plan to execute." },
                "executor": { "type": "string", "description": "The specialist agent to execute steps (e.g., coder, researcher)." },
                "max_iterations": { "type": "integer", "description": "Maximum execution steps in this session." }
            },
            "required": ["name"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: PlanExecuteArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let normalized_name = args.name.replace(" ", "-");
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
            let task_val = get_tool.execute(ctx.clone(), json!({"task_id": normalized_name})).await?;
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
                    "task_id": normalized_name,
                    "status": "done"
                })).await?;
                break;
            };

            let (step_desc, step_criteria, step_method) = {
                let step = &task.steps[idx];
                (step.description.clone(), step.verification_criteria.clone(), step.verification_method.clone())
            };
            log::info!("Plan Execution Iteration {}: Executing step: {}", iterations, step_desc);

            let total_steps = task.steps.len();
            let completed_steps = task.steps.iter().filter(|s| s.completed).count();
            let pct = if total_steps > 0 { (completed_steps as f64 / total_steps as f64 * 100.0) as usize } else { 0 };
            
            println!("\r\n{}", crossterm::style::style(format!("🚀 Plan Execution Progress: [{}%] ({} of {} steps)", pct, completed_steps, total_steps)).cyan().bold());
            let filled_width = pct / 10;
            let bar = format!("[{}{}]", "■".repeat(filled_width), " ".repeat(10 - filled_width));
            println!("   {} Executing step {}: {}\r\n", 
                crossterm::style::style(bar).green(),
                idx + 1,
                crossterm::style::style(&step_desc).yellow().italic()
            );

            // 3. EXECUTE
            let exec_prompt = format!(
                "GOAL: {}\nSTEP: {}\nLAST FEEDBACK: {}\n\nPlease complete this step.",
                task.goal,
                step_desc,
                task.last_step.as_deref().unwrap_or("None")
            );

            let exec_res = executor.execute(ctx.clone(), json!({"input": exec_prompt})).await?;
            let exec_output = exec_res.to_string();

            // 4. VERIFY USING ADVANCED METHODS
            let (is_verified, verify_reason) = if let Some(method) = step_method {
                match method {
                    VerificationMethod::Command { command, expected_output, allow_failure } => {
                        log::info!("Executing verification command: {}", command);
                        let mut cmd = if cfg!(target_os = "windows") {
                            let mut c = tokio::process::Command::new("powershell");
                            c.arg("-Command").arg(&command);
                            c
                        } else {
                            let mut c = tokio::process::Command::new("sh");
                            c.arg("-c").arg(&command);
                            c
                        };
                        match cmd.output().await {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                                let code = output.status.code().unwrap_or(-1);
                                log::info!("Verification command finished with exit code: {}", code);
                                
                                let status_ok = output.status.success() || allow_failure;
                                let output_ok = if let Some(expected) = expected_output {
                                    stdout.contains(&expected) || stderr.contains(&expected)
                                } else {
                                    true
                                };
                                
                                if status_ok && output_ok {
                                    (true, format!("Command successfully verified step.\nExit code: {}\nStdout: {}", code, stdout))
                                } else {
                                    let mut reason = format!("Command verification failed.\nExit code: {}\nStdout: {}\nStderr: {}", code, stdout, stderr);
                                    if !output_ok {
                                        reason.push_str("\nExpected output pattern not found.");
                                    }
                                    (false, reason)
                                }
                            }
                            Err(e) => {
                                (false, format!("Failed to run verification command: {}", e))
                            }
                        }
                    }
                    VerificationMethod::FileExists { path, contains_pattern } => {
                        log::info!("Checking file existence: {}", path);
                        let workspace_root = get_workspace_dir().await.unwrap_or_else(|_| std::path::PathBuf::from("."));
                        let full_path = workspace_root.join(&path);
                        if full_path.exists() {
                            if let Some(pattern) = contains_pattern {
                                match fs::read_to_string(&full_path).await {
                                    Ok(content) => {
                                        if content.contains(&pattern) {
                                            (true, format!("File '{}' exists and correctly contains expected pattern '{}'.", path, pattern))
                                        } else {
                                            (false, format!("File '{}' exists but does not contain expected pattern '{}'.", path, pattern))
                                        }
                                    }
                                    Err(e) => {
                                        (false, format!("File '{}' exists but could not be read: {}", path, e))
                                    }
                                }
                            } else {
                                (true, format!("File '{}' exists.", path))
                            }
                        } else {
                            (false, format!("Expected file '{}' was not found on disk.", path))
                        }
                    }
                    VerificationMethod::Llm { criteria } => {
                        run_llm_verification(&self.model, verifier, ctx.clone(), &task.goal, &step_desc, Some(&criteria), &exec_output).await
                    }
                }
            } else {
                run_llm_verification(&self.model, verifier, ctx.clone(), &task.goal, &step_desc, step_criteria.as_deref(), &exec_output).await
            };

            // 5. UPDATE STATE AND DYNAMICALLY REPLAN IF FAILED
            let update_tool = UpdateTask {};
            if is_verified {
                log::info!("Plan Execution Iteration {}: Step VERIFIED", iterations);
                println!("   {} {}\n", 
                    crossterm::style::style("✓").green().bold(),
                    crossterm::style::style("Step successfully verified!").green()
                );
                task.steps[idx].completed = true;
                update_tool.execute(ctx.clone(), json!({
                    "task_id": normalized_name,
                    "steps": task.steps,
                    "last_step": format!("Step '{}' verified successfully.", step_desc)
                })).await?;
                log_entries.push(format!("Step {}: VERIFIED", idx + 1));
            } else {
                log::info!("Plan Execution Iteration {}: Step FAILED verification. Initiating dynamic replanning...", iterations);
                println!("   {} {}\n", 
                    crossterm::style::style("✗").red().bold(),
                    crossterm::style::style("Step verification failed. Initiating dynamic replanning...").red()
                );
                
                let replan_prompt = format!(
                    r#"You are a high-level Planner. A step in the execution of the high-level goal has FAILED verification.
                    Your task is to adaptively replan the remaining steps to ensure the high-level goal is successfully met.

                    High-Level Goal: {}

                    Current Steps Status:
                    {}

                    Failed Step: "{}"
                    Verification Criteria: "{}"
                    Execution Output:
                    {}

                    Verification Failure Feedback:
                    {}

                    Based on this failure and current progress, output a revised sequence of REMAINING steps to complete the goal.
                    You may keep some future steps unchanged, modify them, or insert new recovery/corrective steps.
                    Only output the remaining incomplete/new steps. Do NOT include steps that are already completed successfully.

                    Return a JSON array of objects: [{{"description": "...", "verification_criteria": "..."}}]"#,
                    task.goal,
                    serde_json::to_string_pretty(&task.steps).unwrap_or_default(),
                    step_desc,
                    step_criteria.as_deref().unwrap_or(""),
                    exec_output,
                    verify_reason
                );

                // Call the Planner model to replan
                let mut replan_stream = self.model.generate_content(
                    LlmRequest::new("planner", vec![Content::new("user").with_text(replan_prompt.clone())]),
                    false,
                ).await.map_err(|e| AdkError::tool(format!("Planner replan request failed: {}", e)))?;

                let mut replan_text = String::new();
                while let Some(event) = replan_stream.next().await {
                    let event = event.map_err(|e| AdkError::tool(format!("Stream error: {}", e)))?;
                    if let Some(content) = event.content {
                        for part in content.parts {
                            if let Some(t) = part.text() {
                                replan_text.push_str(t);
                            }
                        }
                    }
                }

                let completed_steps: Vec<crate::tools::state_manager::Step> = task.steps.iter()
                    .take(idx)
                    .cloned()
                    .collect();

                let replan_res_val = match parse_json_with_healing(
                    &self.model,
                    &replan_text,
                    "planner",
                    &replan_prompt
                ).await {
                    Ok(val) => val,
                    Err(e) => {
                        log::error!("Failed to parse replan JSON, fallback to keeping original plan. Error: {}", e);
                        json!([])
                    }
                };

                let mut parsed_new_steps = Vec::new();
                if let Some(arr) = replan_res_val.as_array() {
                    for v in arr {
                        if let Some(desc) = v.get("description").and_then(|d| d.as_str()) {
                            let criteria = v.get("verification_criteria").and_then(|c| c.as_str()).map(|s| s.to_string());
                            let method: Option<VerificationMethod> = v.get("verification_method")
                                .and_then(|m| serde_json::from_value(m.clone()).ok());
                            parsed_new_steps.push(crate::tools::state_manager::Step {
                                description: desc.to_string(),
                                completed: false,
                                verification_criteria: criteria,
                                verification_method: method,
                            });
                        }
                    }
                }

                if !parsed_new_steps.is_empty() {
                    let mut final_steps = completed_steps;
                    final_steps.extend(parsed_new_steps);
                    
                    log::info!("Successfully replanned task. New plan has {} total steps (including completed).", final_steps.len());
                    
                    update_tool.execute(ctx.clone(), json!({
                        "task_id": normalized_name,
                        "steps": final_steps,
                        "last_step": format!("Verification failed for '{}'. Dynamic replanning triggered. Updated future steps based on feedback: {}", step_desc, verify_reason)
                    })).await?;
                } else {
                    update_tool.execute(ctx.clone(), json!({
                        "task_id": normalized_name,
                        "last_step": format!("Verification failed for '{}'. Replanning failed to yield new steps. Error: {}", step_desc, verify_reason)
                    })).await?;
                }

                log_entries.push(format!("Step {}: FAILED - {}", idx + 1, verify_reason));
            }
        }

        Ok(json!({
            "status": "completed",
            "iterations": iterations,
            "log": log_entries
        }))
    }
}

pub struct PlanGrill;

impl PlanGrill {
    pub async fn generate_questions(model: &Arc<dyn Llm>, objective: &str) -> std::result::Result<Vec<String>, AdkError> {
        let prompt = format!(
            r#"You are a meticulous Project Planner. A user wants to achieve the following goal: "{}"

            Before creating the project/implementation plan, you need to clarify some key details.
            Generate 3 to 5 highly relevant, concise clarification questions that will help design an accurate plan.
            Keep the questions highly focused and direct.

            Return a JSON array of strings: ["Question 1", "Question 2", ...]"#,
            objective
        );

        let mut stream = model.generate_content(
            LlmRequest::new("planner", vec![Content::new("user").with_text(prompt.clone())]),
            false,
        ).await.map_err(|e| AdkError::tool(format!("Llm request failed: {}", e)))?;

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

        let val = parse_json_with_healing(model, &text, "planner", &prompt).await?;
        let mut questions = Vec::new();
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let Some(q) = item.as_str() {
                    questions.push(q.to_string());
                }
            }
        }

        if questions.is_empty() {
            return Err(AdkError::tool("Failed to generate any valid questions."));
        }

        Ok(questions)
    }

    pub async fn synthesize_plan(model: &Arc<dyn Llm>, objective: &str, qa: &[(String, String)]) -> std::result::Result<Vec<Value>, AdkError> {
        let mut qa_text = String::new();
        for (q, a) in qa {
            qa_text.push_str(&format!("Question: {}\nAnswer: {}\n\n", q, a));
        }

        let prompt = format!(
            r#"You are a high-level Project Planner. Break down the following goal into a sequence of precise, executable steps, incorporating the user's clarification answers.

            Goal: {}

            User Clarifications:
            {}

            For EACH step, you MUST provide:
            1. description: What needs to be done.
            2. verification_criteria: Clear, objective criteria to judge if the step is finished correctly.

            Return a JSON array of objects: [{{"description": "...", "verification_criteria": "..."}}]"#,
            objective,
            qa_text
        );

        let mut stream = model.generate_content(
            LlmRequest::new("planner", vec![Content::new("user").with_text(prompt.clone())]),
            false,
        ).await.map_err(|e| AdkError::tool(format!("Llm request failed: {}", e)))?;

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

        let val = parse_json_with_healing(model, &text, "planner", &prompt).await?;
        let mut steps = Vec::new();
        if let Some(arr) = val.as_array() {
            for item in arr {
                steps.push(json!({
                    "description": item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    "verification_criteria": item.get("verification_criteria").and_then(|v| v.as_str()).unwrap_or("Use your best judgment.")
                }));
            }
        }

        if steps.is_empty() {
            return Err(AdkError::tool("Failed to synthesize plan steps."));
        }

        Ok(steps)
    }
}

pub fn plan_tools(model: Arc<dyn Llm>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(PlanCreate::new(model)),
        Arc::new(PlanShow),
        Arc::new(PlanList),
        Arc::new(PlanDelete),
        Arc::new(PlanUpdate),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_tool::SimpleToolContext;

    #[tokio::test]
    async fn test_plan_lifecycle() {
        let ctx = Arc::new(SimpleToolContext::new("test"));
        let plan_name = format!("my-test-plan-{}", uuid::Uuid::new_v4());
        
        // 1. Create a plan without model (fallback steps will be used)
        let create_args = json!({
            "name": plan_name,
            "objective": "test objective"
        });
        let create_tool = PlanCreate { model: None };
        let create_res = create_tool.execute(ctx.clone(), create_args).await.unwrap();
        assert_eq!(create_res["status"], "success");

        // 2. Show the plan
        let show_args = json!({"name": plan_name});
        let show_tool = PlanShow;
        let show_res = show_tool.execute(ctx.clone(), show_args).await.unwrap();
        assert!(show_res["content"].as_str().unwrap().contains(&plan_name));

        // 3. Delete the plan
        let del_args = json!({"name": plan_name});
        let del_tool = PlanDelete;
        let del_res = del_tool.execute(ctx.clone(), del_args).await.unwrap();
        assert_eq!(del_res["status"], "success");
    }
}
