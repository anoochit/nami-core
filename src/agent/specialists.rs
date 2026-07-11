use adk_rust::prelude::*;
use adk_tool::AgentTool;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use serde_json::Value;
use async_trait::async_trait;

pub struct SpecialistSubagentTool {
    inner_agent: Arc<dyn Agent>,
    workspace_mode: String, // "inherit", "branch", "share"
}

impl SpecialistSubagentTool {
    pub fn new(inner_agent: Arc<dyn Agent>, workspace_mode: Option<String>) -> Self {
        Self {
            inner_agent,
            workspace_mode: workspace_mode.unwrap_or_else(|| "inherit".to_string()).to_lowercase(),
        }
    }

    async fn setup_branch_workspace(&self) -> std::result::Result<Option<PathBuf>, String> {
        if self.workspace_mode != "branch" {
            return Ok(None);
        }

        let main_ws = std::env::current_dir()
            .map_err(|e| format!("Failed to get current dir: {}", e))?;
        
        let subagents_dir = main_ws.join("workspace").join(".subagents");
        let id = uuid::Uuid::new_v4().to_string();
        let branched_ws = subagents_dir.join(format!("{}_{}", self.inner_agent.name(), &id[..8]));

        tokio::fs::create_dir_all(&branched_ws)
            .await
            .map_err(|e| format!("Failed to create branched workspace dir: {}", e))?;

        // Copy source/text files to the branched workspace (skipping target, .git, and .subagents)
        let mut entries = tokio::fs::read_dir(&main_ws)
            .await
            .map_err(|e| format!("Failed to read current dir: {}", e))?;

        while let Some(entry) = entries.next_entry().await.ok().flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with('.') || 
               file_name_str == "target" || 
               file_name_str == "workspace" || 
               file_name_str == "node_modules" {
                continue;
            }
            let dest_path = branched_ws.join(&file_name);
            let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
            if metadata.is_file() {
                tokio::fs::copy(entry.path(), dest_path)
                    .await
                    .map_err(|e| format!("Failed to copy file {:?}: {}", file_name, e))?;
            }
        }

        Ok(Some(branched_ws))
    }
}

#[async_trait]
impl Tool for SpecialistSubagentTool {
    fn name(&self) -> &str {
        self.inner_agent.name()
    }

    fn description(&self) -> &str {
        self.inner_agent.description()
    }

    async fn execute(
        &self,
        context: Arc<dyn ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let branch_ws = self.setup_branch_workspace().await
            .map_err(|e| AdkError::tool(format!("Workspace Setup Error: {}", e)))?;

        // Save original NAMI_WORKSPACE
        let original_ws = std::env::var("NAMI_WORKSPACE").ok();

        if let Some(ref path) = branch_ws {
            unsafe {
                std::env::set_var("NAMI_WORKSPACE", path);
            }
        }

        // Execute the agent task
        let execution_result = AgentTool::new(self.inner_agent.clone()).execute(context, args).await;

        // Restore original NAMI_WORKSPACE
        if let Some(ref orig) = original_ws {
            unsafe {
                std::env::set_var("NAMI_WORKSPACE", orig);
            }
        } else {
            unsafe {
                std::env::remove_var("NAMI_WORKSPACE");
            }
        }

        execution_result
    }
}

/// Returns a map of available specialist agents.
///
/// Each specialist is wrapped as a `Tool` to be used by the main agent.
///
/// # Arguments
///
/// * `default_model` - The fallback LLM model.
/// * `specific_models` - Individual models for each specialist.
/// * `tools` - A list of tools to be made available to the specialists.
pub fn get_specialists(
    default_model: Arc<dyn Llm>,
    specific_models: std::collections::HashMap<String, Arc<dyn Llm>>,
    tools: Vec<Arc<dyn Tool>>,
    custom_specs: Option<HashMap<String, super::agent::CustomSpecialistConfig>>,
    skills: Option<adk_rust::skill::SkillIndex>,
    mcp_toolset: Option<Arc<dyn Toolset>>,
) -> HashMap<String, Arc<dyn Tool>> {
    let get_model = |name: &str| {
        specific_models
            .get(name)
            .cloned()
            .unwrap_or_else(|| default_model.clone())
    };

    let mut generalist_builder = LlmAgentBuilder::new("generalist")
        .description(
            "A high-efficiency agent with access to all tools. Use this for repetitive batch tasks or high-volume data processing to keep the main conversation history lean."
        )
        .instruction(
            "You are a generalist agent. Perform the requested batch tasks or data processing efficiently."
        )
        .model(get_model("generalist"));

    for t in &tools {
        generalist_builder = generalist_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        generalist_builder = generalist_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        generalist_builder = generalist_builder.toolset(m.clone());
    }
    let generalist = Arc::new(
        generalist_builder
            .build()
            .expect("Failed to build generalist agent"),
    );

    let mut coder_builder = LlmAgentBuilder::new("coder")
        .description(
            "A specialist in software engineering, debugging, and code refactoring. Use this for complex coding tasks."
        )
        .instruction(
            "You are an expert software engineer. Provide clean, efficient, and well-documented code solutions. Focus on best practices and system integrity."
        )
        .model(get_model("coder"));

    for t in &tools {
        coder_builder = coder_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        coder_builder = coder_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        coder_builder = coder_builder.toolset(m.clone());
    }
    let coder = Arc::new(coder_builder.build().expect("Failed to build coder agent"));

    let mut researcher_builder = LlmAgentBuilder::new("researcher")
        .description(
            "A specialist in information retrieval, documentation analysis, and data synthesis. Use this for deep-dive research tasks."
        )
        .instruction(
            "You are a deep-dive researcher. Analyze information meticulously, identify key insights, and provide comprehensive summaries based on available data."
        )
        .model(get_model("researcher"));

    for t in &tools {
        researcher_builder = researcher_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        researcher_builder = researcher_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        researcher_builder = researcher_builder.toolset(m.clone());
    }
    let researcher = Arc::new(
        researcher_builder
            .build()
            .expect("Failed to build researcher agent"),
    );

    let mut writer_builder = LlmAgentBuilder::new("writer")
        .description(
            "A specialist in technical writing, content creation, and professional communication. Use this for drafting documents and reports."
        )
        .instruction(
            "You are a professional technical writer. Craft clear, engaging, and well-structured content tailored to the requested audience and format."
        )
        .model(get_model("writer"));

    for t in &tools {
        writer_builder = writer_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        writer_builder = writer_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        writer_builder = writer_builder.toolset(m.clone());
    }
    let writer = Arc::new(
        writer_builder
            .build()
            .expect("Failed to build writer agent"),
    );

    let mut ralph_builder = LlmAgentBuilder::new("ralph")
        .description(
            "A playful and persistent autonomous agent that runs in a loop to achieve a goal. It doesn't give up!"
        )
        .instruction(
            "You are Ralph Wiggum. You are simple, literal, and very persistent. You might say silly things, but you never stop trying to reach your goal. When you are done, say 'I'm a winner!'"
        )
        .model(get_model("ralph"));

    for t in &tools {
        ralph_builder = ralph_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        ralph_builder = ralph_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        ralph_builder = ralph_builder.toolset(m.clone());
    }
    let ralph = Arc::new(ralph_builder.build().expect("Failed to build ralph agent"));

    let mut verifier_builder = LlmAgentBuilder::new("verifier")
        .description(
            "A rigorous evaluation specialist. It analyzes outputs against verification criteria and identifies faults, edge cases, or missed requirements."
        )
        .instruction(
            "You are a rigorous Verifier. Your goal is to ensure that the work performed by other agents is correct, complete, and meets all verification criteria.\n\
            Be critical. Look for bugs, side effects, or incomplete implementations.\n\
            You have access to filesystem tools. If a step involves creating, editing, or verifying a file (such as `site.html`), you MUST use your tools to inspect the file's contents on disk instead of guessing.\n\
            You MUST respond with a JSON object of this structure:\n\
            {\n\
              \"verified\": true or false,\n\
              \"reasoning\": \"Your detailed reasoning here\",\n\
              \"suggested_fixes\": \"Suggestions for the executor if not verified\"\n\
            }"
        )
        .model(get_model("verifier"));

    for t in &tools {
        verifier_builder = verifier_builder.tool(t.clone());
    }
    if let Some(ref s) = skills {
        verifier_builder = verifier_builder.with_skills(s.clone());
    }
    if let Some(ref m) = mcp_toolset {
        verifier_builder = verifier_builder.toolset(m.clone());
    }
    let verifier = Arc::new(verifier_builder.build().expect("Failed to build verifier agent"));

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert(
        "generalist".to_string(),
        Arc::new(SpecialistSubagentTool::new(generalist, None)),
    );
    specialists.insert("coder".to_string(), Arc::new(SpecialistSubagentTool::new(coder, None)));
    specialists.insert(
        "researcher".to_string(),
        Arc::new(SpecialistSubagentTool::new(researcher, None)),
    );
    specialists.insert("writer".to_string(), Arc::new(SpecialistSubagentTool::new(writer, None)));
    specialists.insert("ralph".to_string(), Arc::new(SpecialistSubagentTool::new(ralph, None)));
    specialists.insert("verifier".to_string(), Arc::new(SpecialistSubagentTool::new(verifier, None)));

    if let Some(specs) = custom_specs {
        for (name, config) in specs {
            let mut agent_builder = LlmAgentBuilder::new(&name)
                .description(&config.description)
                .instruction(&config.instruction)
                .model(get_model(&name));
            for t in &tools {
                if let Some(ref allowed) = config.tools {
                    if allowed.iter().any(|name| name == t.name()) {
                        agent_builder = agent_builder.tool(t.clone());
                    }
                } else {
                    agent_builder = agent_builder.tool(t.clone());
                }
            }
            if let Some(ref s) = skills {
                agent_builder = agent_builder.with_skills(s.clone());
            }
            if let Some(ref m) = mcp_toolset {
                agent_builder = agent_builder.toolset(m.clone());
            }
            match agent_builder.build() {
                Ok(agent) => {
                    let wrapped_agent = Arc::new(agent);
                    specialists.insert(
                        name.clone(), 
                        Arc::new(SpecialistSubagentTool::new(wrapped_agent, config.workspace_mode.clone()))
                    );
                }
                Err(e) => {
                    log::error!("Failed to build custom specialist agent '{}': {}", name, e);
                }
            }
        }
    }

    specialists
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_tool::SimpleToolContext;
    use serde_json::json;

    struct MockAgent;

    #[async_trait]
    impl Agent for MockAgent {
        fn name(&self) -> &str {
            "mock_agent"
        }

        fn description(&self) -> &str {
            "A test mock agent"
        }

        fn sub_agents(&self) -> &[Arc<dyn Agent>] {
            &[]
        }

        async fn run(
            &self,
            _ctx: Arc<dyn InvocationContext>,
        ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<adk_session::Event>> + Send + 'static>>> {
            let event = adk_session::Event {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                invocation_id: "".to_string(),
                branch: "".to_string(),
                author: "".to_string(),
                actions: Default::default(),
                llm_response: LlmResponse {
                    content: Some(Content::new("model").with_text("mock agent response")),
                    ..Default::default()
                },
                llm_request: Default::default(),
                long_running_tool_ids: Default::default(),
                provider_metadata: Default::default(),
            };
            let stream = futures::stream::once(async move { Ok(event) });
            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_specialist_tool_metadata() {
        let mock_agent = Arc::new(MockAgent);
        let tool = SpecialistSubagentTool::new(mock_agent, None);

        assert_eq!(tool.name(), "mock_agent");
        assert_eq!(tool.description(), "A test mock agent");
    }

    #[tokio::test]
    async fn test_specialist_tool_workspace_inherit() {
        let mock_agent = Arc::new(MockAgent);
        let tool = SpecialistSubagentTool::new(mock_agent, Some("inherit".to_string()));

        let path = tool.setup_branch_workspace().await;
        assert!(path.is_ok());
        assert!(path.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_specialist_tool_execution() {
        let mock_agent = Arc::new(MockAgent);
        let tool = SpecialistSubagentTool::new(mock_agent, None);

        let ctx = Arc::new(SimpleToolContext::new("test_caller"));
        let args = json!({ "input": "test prompt" });

        let result = tool.execute(ctx, args).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.to_string().contains("mock agent response"));
    }
}

