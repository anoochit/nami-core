use adk_rust::prelude::*;
use adk_tool::AgentTool;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use serde_json::Value;
use async_trait::async_trait;
use futures::StreamExt;

pub struct StreamSpecialistAgent {
    inner_agent: Arc<dyn Agent>,
}

impl StreamSpecialistAgent {
    pub fn new(inner_agent: Arc<dyn Agent>) -> Self {
        Self { inner_agent }
    }
}

#[async_trait]
impl Agent for StreamSpecialistAgent {
    fn name(&self) -> &str {
        self.inner_agent.name()
    }

    fn description(&self) -> &str {
        self.inner_agent.description()
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        self.inner_agent.sub_agents()
    }

    async fn run(
        &self,
        ctx: Arc<dyn InvocationContext>,
    ) -> Result<std::pin::Pin<std::boxed::Box<dyn futures::Stream<Item = Result<adk_session::Event>> + std::marker::Send + 'static>>> {
        let stream = self.inner_agent.run(ctx).await?;
        let agent_name = self.inner_agent.name().to_string();

        let mapped_stream = stream.map(move |res| {
            if let Ok(ref event) = res {
                if let Some(ref content) = event.llm_response.content {
                    for part in &content.parts {
                        match part {
                            Part::Thinking { thinking, .. } => {
                                if !thinking.is_empty() {
                                    use std::io::Write;
                                    let formatted_thinking = thinking.replace('\n', "\r\n        ");
                                    print!(
                                        "\r\n\x1b[38;2;189;147;249m └─\x1b[0m [\x1b[38;2;139;233;253m{}\x1b[0m] \x1b[3m\x1b[2m🧠 {}\x1b[0m\r\n",
                                        agent_name, formatted_thinking
                                    );
                                    let _ = std::io::stdout().flush();
                                }
                            }
                            Part::FunctionCall { name, args, .. } => {
                                use std::io::Write;
                                let raw_args = args.to_string();
                                let minified_args = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw_args) {
                                    serde_json::to_string(&val).unwrap_or_else(|_| raw_args.clone())
                                } else {
                                    raw_args.clone()
                                };

                                let formatted_args = if minified_args.chars().count() > 80 {
                                    let truncated: String = minified_args.chars().take(80).collect();
                                    format!("{}... (+{} chars)", truncated, minified_args.len() - truncated.len())
                                } else {
                                    minified_args
                                };

                                print!(
                                    "\r\n\x1b[38;2;189;147;249m └─\x1b[0m [\x1b[38;2;139;233;253m{}\x1b[0m] 🔧 \x1b[38;2;255;121;198mCalling tool:\x1b[0m \x1b[1m{}\x1b[0m with args: {}\r\n",
                                    agent_name, name, formatted_args
                                );
                                let _ = std::io::stdout().flush();
                            }
                            _ => {}
                        }
                    }
                }
            }
            res
        });

        Ok(Box::pin(mapped_stream))
    }
}

pub struct SpecialistSubagentTool {
    inner_agent: Arc<dyn Agent>,
    workspace_mode: String, // "inherit", "branch", "share"
}

impl SpecialistSubagentTool {
    pub fn new(inner_agent: Arc<dyn Agent>, workspace_mode: Option<String>) -> Self {
        Self {
            inner_agent: Arc::new(StreamSpecialistAgent::new(inner_agent)),
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

struct SpecialistDefinition {
    name: &'static str,
    description: &'static str,
    instruction: &'static str,
}

const BUILTIN_SPECIALISTS: &[SpecialistDefinition] = &[
    SpecialistDefinition {
        name: "generalist",
        description: "A high-efficiency agent with access to all tools. Use this for repetitive batch tasks or high-volume data processing to keep the main conversation history lean.",
        instruction: "You are a generalist agent. Perform the requested batch tasks or data processing efficiently.",
    },
    SpecialistDefinition {
        name: "coder",
        description: "A specialist in software engineering, debugging, and code refactoring. Use this for complex coding tasks.",
        instruction: "You are an expert software engineer. Provide clean, efficient, and well-documented code solutions. Focus on best practices and system integrity.",
    },
    SpecialistDefinition {
        name: "researcher",
        description: "A specialist in information retrieval, documentation analysis, and data synthesis. Use this for deep-dive research tasks.",
        instruction: "You are a deep-dive researcher. Analyze information meticulously, identify key insights, and provide comprehensive summaries based on available data.",
    },
    SpecialistDefinition {
        name: "writer",
        description: "A specialist in technical writing, content creation, and professional communication. Use this for drafting documents and reports.",
        instruction: "You are a professional technical writer. Craft clear, engaging, and well-structured content tailored to the requested audience and format.",
    },
    SpecialistDefinition {
        name: "ralph",
        description: "A playful and persistent autonomous agent that runs in a loop to achieve a goal. It doesn't give up!",
        instruction: "You are Ralph Wiggum. You are simple, literal, and very persistent. You might say silly things, but you never stop trying to reach your goal. When you are done, say 'I'm a winner!'",
    },
    SpecialistDefinition {
        name: "verifier",
        description: "A rigorous evaluation specialist. It analyzes outputs against verification criteria and identifies faults, edge cases, or missed requirements.",
        instruction: "You are a rigorous Verifier. Your goal is to ensure that the work performed by other agents is correct, complete, and meets all verification criteria.\n\
            Be critical. Look for bugs, side effects, or incomplete implementations.\n\
            You have access to filesystem tools. If a step involves creating, editing, or verifying a file (such as `site.html`), you MUST use your tools to inspect the file's contents on disk instead of guessing.\n\
            You MUST respond with a JSON object of this structure:\n\
            {\n\
              \"verified\": true or false,\n\
              \"reasoning\": \"Your detailed reasoning here\",\n\
              \"suggested_fixes\": \"Suggestions for the executor if not verified\"\n\
            }",
    },
    SpecialistDefinition {
        name: "designer",
        description: "A specialist in high-fidelity web design, frontend interfaces, and interactive prototyping. Uses utility-first Tailwind CSS, custom dark modes, premium typography, and responsive grids.",
        instruction: "You are a world-class Web Designer and Frontend Engineer. Your goal is to create stunning, premium, responsive web pages and interactive UI layouts.\n\
            Always enforce the following visual excellence standards:\n\
            1. **Tailwind CSS Utility-First**: Rely primarily on utility-first Tailwind CSS (loaded via modern CDN script) directly in your HTML for fast, visual, and modular layout styling. Do not write extensive ad-hoc inline styles.\n\
            2. **Premium Visual Aesthetics**: Create designs that wow users immediately. Use dark mode-first palettes (sleek grays, curated purples/emeralds/cyans), smooth linear/radial gradients, and glassmorphism (backdrop-blur, semi-transparent borders).\n\
            3. **Aesthetic Typography**: Use modern web fonts (e.g., Google Fonts like Inter, Outfit, or Playfair Display). Ensure crisp font-sizes, responsive line heights, and perfect leading.\n\
            4. **Smooth Interactions**: Implement hover effects, transitions, subtle micro-animations (scale-up on hover, fading, sliding elements), and responsive burger-menus or dropdowns.\n\
            5. **Clean Structure**: Use standard HTML5 semantic elements (header, main, nav, section, footer) with robust modular layouts.",
    },
];

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

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();

    // Dynamically build and register all built-in specialists
    for spec in BUILTIN_SPECIALISTS {
        let mut builder = LlmAgentBuilder::new(spec.name)
            .description(spec.description)
            .instruction(spec.instruction.to_string())
            .model(get_model(spec.name));

        for t in &tools {
            builder = builder.tool(t.clone());
        }
        if let Some(ref s) = skills {
            builder = builder.with_skills(s.clone());
        }
        if let Some(ref m) = mcp_toolset {
            builder = builder.toolset(m.clone());
        }

        match builder.build() {
            Ok(agent) => {
                specialists.insert(
                    spec.name.to_string(),
                    Arc::new(SpecialistSubagentTool::new(Arc::new(agent), None)),
                );
            }
            Err(e) => {
                log::error!("Failed to build specialist agent '{}': {}", spec.name, e);
            }
        }
    }

    if let Some(specs) = custom_specs {
        for (name, config) in specs {
            let mut agent_builder = LlmAgentBuilder::new(&name)
                .description(&config.description)
                .instruction(&config.instruction)
                .model(get_model(&name));
            for t in &tools {
                if let Some(ref allowed) = config.tools {
                    if allowed.iter().any(|n| n == t.name()) {
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

    #[test]
    fn test_custom_specialist_deserialization() {
        let toml_str = r#"
            [model]
            provider = "gemini"
            model_name = "gemini-2.5-flash"

            [specialists.custom.database_guru]
            description = "A database expert"
            instruction = "Solve queries"
            provider = "openai"
            model_name = "gpt-4"
        "#;
        let config: crate::agent::AppConfig = toml::from_str(toml_str).unwrap();
        assert!(config.specialists.is_some());
        let specs = config.specialists.unwrap();
        assert!(specs.custom.is_some());
        let custom = specs.custom.unwrap();
        assert!(custom.contains_key("database_guru"));
        let guru = custom.get("database_guru").unwrap();
        assert_eq!(guru.description, "A database expert");
        assert_eq!(guru.instruction, "Solve queries");
        assert_eq!(guru.provider.as_deref(), Some("openai"));
        assert_eq!(guru.model_name.as_deref(), Some("gpt-4"));
    }
}


