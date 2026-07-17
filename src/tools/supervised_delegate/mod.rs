use adk_rust::Tool;
use adk_rust::prelude::*;
use adk_rust::serde::Deserialize;
use futures::future::join_all;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::StreamExt;

#[derive(Deserialize, Debug, Clone)]
pub struct Subtask {
    pub id: String,
    pub specialist: String,
    pub prompt: String,
    pub dependencies: Vec<String>,
}

#[derive(Deserialize)]
pub struct SupervisedDelegateArgs {
    pub task: String,
    pub max_refinement_turns: Option<usize>,
}

pub struct SupervisedDelegate {
    model: Arc<dyn Llm>,
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl SupervisedDelegate {
    pub fn new(model: Arc<dyn Llm>, specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { model, specialists }
    }

    /// Helper to make batch completions with the main model.
    async fn prompt_llm(&self, prompt: &str) -> std::result::Result<String, String> {
        Self::static_prompt_llm(self.model.clone(), prompt).await
    }

    async fn static_prompt_llm(model: Arc<dyn Llm>, prompt: &str) -> std::result::Result<String, String> {
        let mut stream = model.generate_content(
            LlmRequest::new(
                model.name().to_string(),
                vec![Content::new("user").with_text(prompt)],
            ),
            false,
        ).await.map_err(|e| format!("LLM generation failed: {}", e))?;

        let mut content = String::new();
        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| e.to_string())?;
            if let Some(c) = event.content {
                for part in c.parts {
                    if let Some(t) = part.text() {
                        content.push_str(t);
                    }
                }
            }
        }
        Ok(content)
    }
}

#[async_trait::async_trait]
impl Tool for SupervisedDelegate {
    fn name(&self) -> &str {
        "supervised_delegate"
    }

    fn description(&self) -> &str {
        "Runs an advanced multi-agent supervisory routing flow. It splits a complex task into a DAG of subtasks, runs independent subtasks concurrently via specialized sub-agents, verifies their work, and synthesizes the outputs."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "task": { "type": "string", "description": "The complex multi-step task or project to delegate." },
                "max_refinement_turns": { "type": "integer", "description": "Maximum refinement verification loops per subtask. Default is 2." }
            },
            "required": ["task"]
        }))
    }

    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: SupervisedDelegateArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let max_turns = args.max_refinement_turns.unwrap_or(2);

        let mut specs_desc = String::new();
        let mut keys: Vec<String> = self.specialists.keys().cloned().collect();
        keys.sort();
        for key in keys {
            if let Some(tool) = self.specialists.get(&key) {
                specs_desc.push_str(&format!("- '{}': {}\n", key, tool.description()));
            }
        }

        // 1. Planning Step: Split the task into a structured DAG of subtasks
        let planning_prompt = format!(
            r#"You are the Multi-Agent Supervisor Router. Decompose the following user task into a list of specialized subtasks with dependencies (a DAG structure).
Available Specialists:
{}
Task:
"{}"

Return a valid JSON array of subtasks, adhering strictly to this schema:
[
  {{
    "id": "task_id_1",
    "specialist": "researcher",
    "prompt": "Decomposed instructions for the specialist...",
    "dependencies": []
  }},
  {{
    "id": "task_id_2",
    "specialist": "coder",
    "prompt": "Next step instructions...",
    "dependencies": ["task_id_1"]
  }}
]
Return ONLY the raw JSON array. Do not enclose it in markdown code fences or prefix it with comments."#,
            specs_desc, args.task
        );

        let plan_raw = self.prompt_llm(&planning_prompt).await
            .map_err(|e| AdkError::tool(format!("Supervisor Planning failed: {}", e)))?;

        // Sanitize LLM response (strip markdown blocks if present)
        let cleaned_plan = plan_raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let subtasks: Vec<Subtask> = serde_json::from_str(cleaned_plan)
            .map_err(|e| AdkError::tool(format!("Failed to parse supervisor plan: {}. Raw output: {}", e, plan_raw)))?;

        // 2. DAG Execution Loop
        let subtasks_map: HashMap<String, Subtask> = subtasks.iter().map(|s| (l(&s.id), s.clone())).collect();
        let completed_tasks = Arc::new(Mutex::new(HashSet::<String>::new()));
        let task_results = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        let mut active_or_pending: HashSet<String> = subtasks.iter().map(|s| l(&s.id)).collect();

        // Standardize lowercase strings for safety
        fn l(s: &str) -> String {
            s.to_lowercase().trim().to_string()
        }

        while !active_or_pending.is_empty() {
            // Find all tasks that have no unresolved dependencies
            let mut executable_tasks = Vec::new();
            {
                let completed = completed_tasks.lock().await;
                for id in &active_or_pending {
                    if let Some(subtask) = subtasks_map.get(id) {
                        let has_unresolved = subtask.dependencies.iter().any(|dep| {
                            !completed.contains(&l(dep))
                        });
                        if !has_unresolved {
                            executable_tasks.push(subtask.clone());
                        }
                    }
                }
            }

            if executable_tasks.is_empty() {
                if !active_or_pending.is_empty() {
                    return Err(AdkError::tool(format!(
                        "Deadlock or dependency issue in supervisor routing DAG. Unresolved tasks: {:?}",
                        active_or_pending
                    )));
                }
                break;
            }

            // Remove executable tasks from active_or_pending so they aren't scheduled again
            for t in &executable_tasks {
                active_or_pending.remove(&l(&t.id));
            }

            // Execute executable tasks concurrently
            let mut join_handles = Vec::new();
            for task in executable_tasks {
                let completed_tasks_clone = completed_tasks.clone();
                let task_results_clone = task_results.clone();
                let specialists_clone = self.specialists.clone();
                let ctx_clone = ctx.clone();
                let model_clone = self.model.clone();

                join_handles.push(tokio::spawn(async move {
                    let id_lower = l(&task.id);
                    let specialist_name = l(&task.specialist);
                    
                    let mut dependency_outputs = String::new();
                    {
                        let results = task_results_clone.lock().await;
                        for dep in &task.dependencies {
                            let dep_lower = dep.to_lowercase().trim().to_string();
                            if let Some(dep_out) = results.get(&dep_lower) {
                                dependency_outputs.push_str(&format!(
                                    "--- Output of Dependency Subtask '{}' ---\n{}\n\n",
                                    dep, dep_out
                                ));
                            }
                        }
                    }

                    let mut current_prompt = task.prompt.clone();
                    if !dependency_outputs.is_empty() {
                        current_prompt = format!(
                            "You are provided with the outputs of prior subtasks that this task depends on. Use them to build upon or integrate into your work.\n\n{}\nTask instructions:\n{}",
                            dependency_outputs, task.prompt
                        );
                    }

                    let mut final_output = String::new();

                    for turn in 1..=max_turns {
                        // 1. Invoke specialist
                        let mut output = format!("Failed to find specialist '{}'", specialist_name);
                        if let Some(tool) = specialists_clone.get(&specialist_name) {
                            match tool.execute(ctx_clone.clone(), json!({ "input": current_prompt })).await {
                                Ok(val) => {
                                    output = if let Some(s) = val.as_str() {
                                        s.to_string()
                                    } else {
                                        val.to_string()
                                    };
                                }
                                Err(e) => {
                                    output = format!("Execution Error: {}", e);
                                }
                            }
                        }

                        // 2. Verification Step
                        let verification_prompt = format!(
                            r#"You are a strict QA Verifier. Assess if the specialist's output meets the requested prompt criteria.
Requested Subtask:
"{}"

Specialist Output:
"{}"

If the output is correct and complete, reply with:
VERIFIED
If there are faults, bugs, or missing elements, reply with:
REJECTED
Followed by detailed feedback on what needs to be fixed on the next lines."#,
                            task.prompt, output
                        );

                        match Self::static_prompt_llm(model_clone.clone(), &verification_prompt).await {
                            Ok(verify_res) => {
                                if verify_res.trim().starts_with("VERIFIED") {
                                    final_output = output;
                                    break;
                                } else {
                                    // Rejected! Attempt refinement with feedback
                                    current_prompt = format!(
                                        "Your previous attempt was rejected with this feedback. Please correct it:\n{}\n\nTask:\n{}",
                                        verify_res, task.prompt
                                    );
                                    final_output = format!("(Refinement Attempt {}) Rejected: {}", turn, output);
                                }
                            }
                            Err(e) => {
                                final_output = format!("QA verification failure: {}", e);
                                break;
                            }
                        }
                    }

                    // Store results
                    {
                        let mut results = task_results_clone.lock().await;
                        results.insert(id_lower.clone(), final_output);
                    }
                    {
                        let mut completed = completed_tasks_clone.lock().await;
                        completed.insert(id_lower);
                    }
                }));
            }

            // Wait for this round of concurrent subtasks to complete
            join_all(join_handles).await;
        }

        // 3. Final Synthesis Step: Compile all subtask outputs into a cohesive master report
        let mut executed_details = String::new();
        {
            let results = task_results.lock().await;
            for (id, output) in results.iter() {
                executed_details.push_str(&format!("### Subtask: {}\n{}\n\n", id, output));
            }
        }

        let synthesis_prompt = format!(
            r#"You are the Lead Supervisor Agent. Compile a premium, highly detailed synthesis report of the completed project. Integrate the results of all specialized subtasks into a cohesive, structured response for the user.

Primary Task Goal:
"{}"

Executed Subtask Outputs:
{}

Provide a professional, clear, and comprehensive synthesis of the results, next steps, and architectural outcomes. Use markdown alerts or tables where helpful to highlight details."#,
            args.task, executed_details
        );

        let final_report = self.prompt_llm(&synthesis_prompt).await
            .map_err(|e| AdkError::tool(format!("Supervisor Synthesis failed: {}", e)))?;

        Ok(json!({
            "status": "success",
            "report": final_report,
            "subtasks_run": subtasks_map.len()
        }))
    }
}

pub fn supervised_delegate_tool(
    model: Arc<dyn Llm>,
    specialists: HashMap<String, Arc<dyn Tool>>,
) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(SupervisedDelegate::new(model, specialists))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_rust::{Tool, ToolContext};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockSpecialist;
    #[async_trait::async_trait]
    impl Tool for MockSpecialist {
        fn name(&self) -> &str { "mock_specialist" }
        fn description(&self) -> &str { "Mock Specialist Tool" }
        async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
            Ok(json!(format!("Completed: {}", args["input"])))
        }
    }

    struct MockLlm {
        call_count: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl Llm for MockLlm {
        fn name(&self) -> &str {
            "mock-gemini-model"
        }

        async fn generate_content(
            &self,
            _req: LlmRequest,
            _stream: bool,
        ) -> std::result::Result<
            std::pin::Pin<std::boxed::Box<dyn futures::Stream<Item = std::result::Result<LlmResponse, AdkError>> + std::marker::Send>>,
            AdkError,
        > {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            let response_text = match count {
                0 => r#"[
                    {
                        "id": "task_1",
                        "specialist": "coder",
                        "prompt": "Write test code",
                        "dependencies": []
                    }
                ]"#.to_string(),
                1 => "VERIFIED".to_string(),
                _ => "Mock synthesis master report of task_1 success!".to_string(),
            };

            let response = LlmResponse {
                content: Some(Content::new("model").with_text(response_text)),
                ..Default::default()
            };

            let stream = futures::stream::once(async move { Ok(response) });
            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_supervised_delegate_execution() {
        let mock_llm = Arc::new(MockLlm { call_count: AtomicUsize::new(0) });
        let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        specialists.insert("coder".to_string(), Arc::new(MockSpecialist) as Arc<dyn Tool>);

        let delegate_tool = SupervisedDelegate::new(mock_llm, specialists);
        let ctx = Arc::new(adk_tool::SimpleToolContext::new("test_caller"));

        let args = json!({
            "task": "Build a test app",
            "max_refinement_turns": 2
        });

        let result = delegate_tool.execute(ctx, args).await.unwrap();

        assert_eq!(result["status"], "success");
        assert_eq!(result["subtasks_run"], 1);
        assert!(result["report"].as_str().unwrap().contains("Mock synthesis master report"));
    }
}
