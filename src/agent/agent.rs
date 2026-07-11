use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// use std::time::SystemTime;

use super::mcp;
use super::specialists;
use crate::tools;
use crate::utils::get_nami_dir;

// Providers
use adk_rust::model::{OpenAIClient, OpenAIConfig};

/// Application configuration structure loaded from `config.toml`.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AppConfig {
    /// LLM provider configuration.
    pub model: ModelConfig,
    /// Optional configuration for specialized agents.
    pub specialists: Option<SpecialistsConfig>,
    /// Optional configuration for image generation.
    pub image_generation: Option<ModelConfig>,
    /// Optional configuration for audio generation.
    pub audio_generation: Option<ModelConfig>,
    /// Optional configuration for video generation.
    pub video_generation: Option<ModelConfig>,
    /// Optional configuration for reflection service.
    pub reflection: Option<ReflectionConfig>,
    /// Optional configuration for embedding service.
    pub embedding: Option<ModelConfig>,
    /// Optional configuration for tools, like shell command whitelist.
    pub tools: Option<ToolsConfig>,
}

/// Configuration for tools (e.g., shell commands whitelist).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToolsConfig {
    pub shell: Option<ShellToolConfig>,
    pub enabled_categories: Option<Vec<String>>,
}

/// Configuration for the shell tool.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ShellToolConfig {
    pub allowed_commands: Option<Vec<String>>,
    pub blocked_commands: Option<Vec<String>>,
    pub security_level: Option<String>,
    pub sanitize_environment: Option<bool>,
}

/// Configuration details for the LLM provider and specific model.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ModelConfig {
    /// Name of the LLM provider (e.g., "gemini", "anthropic", "vertex", "openai").
    pub provider: Option<String>,
    /// Identifier for the model to use (e.g., "gemini-1.5-pro", "gpt-4o").
    pub model_name: String,
    /// Environment variable name containing the API key for this provider.
    /// Defaults to "GOOGLE_API_KEY" for Gemini if not specified.
    pub api_key_env: Option<String>,
    /// Optional base URL for API requests, useful for compatible providers or local proxies.
    #[allow(dead_code)]
    pub base_url: Option<String>,
    /// Google Cloud Project ID, required for Vertex AI.
    pub project_id: Option<String>,
    /// Google Cloud Location (region), required for Vertex AI.
    pub location: Option<String>,
}

/// Configuration for dynamic custom specialist agents.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CustomSpecialistConfig {
    pub provider: Option<String>,
    pub model_name: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub description: String,
    pub instruction: String,
    pub tools: Option<Vec<String>>,
    pub workspace_mode: Option<String>, // "inherit", "branch", or "share"
}

/// Configuration for individual specialized agents.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SpecialistsConfig {
    /// Configuration for the coding specialist.
    pub coder: Option<ModelConfig>,
    /// Configuration for the research specialist.
    pub researcher: Option<ModelConfig>,
    /// Configuration for the writing specialist.
    pub writer: Option<ModelConfig>,
    /// Configuration for the Ralph Wiggum (recursive) agent.
    pub ralph: Option<ModelConfig>,
    /// Configuration for the generalist agent.
    pub generalist: Option<ModelConfig>,
    /// Additional dynamic custom specialist agents
    #[serde(default)]
    pub custom: Option<HashMap<String, CustomSpecialistConfig>>,
}

impl SpecialistsConfig {
    /// Helper to cleanly load all specialist LLM instances.
    pub async fn load_all_models(
        &self,
        default_config: &ModelConfig,
    ) -> anyhow::Result<HashMap<String, Arc<dyn Llm>>> {
        let mut models = HashMap::new();
        let configs = [
            ("coder", &self.coder),
            ("researcher", &self.researcher),
            ("writer", &self.writer),
            ("ralph", &self.ralph),
            ("generalist", &self.generalist),
        ];

        for (name, cfg) in configs {
            if let Some(cfg) = cfg {
                let model = load_model_with_fallback(&Some(cfg.clone()), default_config).await?;
                models.insert(name.to_string(), model);
            }
        }

        if let Some(ref custom_specs) = self.custom {
            for (name, custom_cfg) in custom_specs {
                let model_cfg = ModelConfig {
                    provider: custom_cfg.provider.clone(),
                    model_name: custom_cfg.model_name.clone().unwrap_or_else(|| default_config.model_name.clone()),
                    api_key_env: custom_cfg.api_key_env.clone(),
                    base_url: custom_cfg.base_url.clone(),
                    project_id: custom_cfg.project_id.clone(),
                    location: custom_cfg.location.clone(),
                };
                let loaded_model = load_model_with_fallback(&Some(model_cfg), default_config).await?;
                models.insert(name.clone(), loaded_model);
            }
        }

        Ok(models)
    }
}


/// Configuration for the reflection service.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ReflectionConfig {
    /// Whether the reflection service is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Name of the LLM provider for the reflection service.
    pub provider: Option<String>,
    /// Identifier for the model to use for reflection.
    pub model_name: Option<String>,
    /// Environment variable name containing the API key.
    pub api_key_env: Option<String>,
    /// Optional base URL for the reflection service.
    pub base_url: Option<String>,
    /// Google Cloud Project ID for Vertex AI reflection.
    pub project_id: Option<String>,
    /// Google Cloud Location for Vertex AI reflection.
    pub location: Option<String>,
}

impl ReflectionConfig {
    /// Converts the reflection configuration into a standard `ModelConfig`.
    pub fn to_model_config(&self) -> Option<ModelConfig> {
        self.model_name.as_ref().map(|name| ModelConfig {
            provider: self.provider.clone(),
            model_name: name.clone(),
            api_key_env: self.api_key_env.clone(),
            base_url: self.base_url.clone(),
            project_id: self.project_id.clone(),
            location: self.location.clone(),
        })
    }
}

/// Synchronously loads the application configuration from `~/.nami/config.toml`.
pub fn load_config_sync() -> anyhow::Result<AppConfig> {
    let config_path = get_nami_dir().join("config.toml");
    let config_str = std::fs::read_to_string(config_path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}

/// Synchronously saves the application configuration to `~/.nami/config.toml`.
pub fn save_config_sync(config: &AppConfig) -> anyhow::Result<()> {
    let config_path = get_nami_dir().join("config.toml");
    let config_str = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, config_str.as_bytes())?;
    Ok(())
}

use adk_rust::skill::SkillIndex;

/// Loads skills from local workspace (`.skills/` and `skills/`) and global directories (`~/.agents/skills/` and `~/.nami/skills/`), prioritizing local workspace skills.
pub fn load_global_skills() -> anyhow::Result<SkillIndex> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let agents_skills_dir = home_dir.join(".agents").join("skills");
    let nami_skills_dir = crate::utils::get_nami_dir().join("skills");

    let mut extra_dirs = Vec::new();
    let local_skills = cwd.join("skills");
    if local_skills.exists() && local_skills.is_dir() {
        extra_dirs.push(local_skills);
    }
    if agents_skills_dir.exists() && agents_skills_dir.is_dir() {
        extra_dirs.push(agents_skills_dir);
    }
    if nami_skills_dir.exists() && nami_skills_dir.is_dir() {
        extra_dirs.push(nami_skills_dir);
    }

    let index = adk_rust::skill::load_skill_index_with_extras(&cwd, &extra_dirs)?;
    Ok(index)
}

/// Dynamically discovers and formats a list of all available global and local skills and descriptions.
pub fn get_global_skills_summary() -> String {
    if let Ok(skills_index) = load_global_skills() {
        let mut summaries = Vec::new();
        for skill in skills_index.skills() {
            summaries.push(format!("- {}: {}", skill.name, skill.description));
        }
        if summaries.is_empty() {
            String::new()
        } else {
            summaries.sort();
            format!(
                "━━━ AVAILABLE AGENT SKILLS ━━━\n{}\n\n",
                summaries.join("\n")
            )
        }
    } else {
        String::new()
    }
}

/// Counts the number of skills in global directories (~/.agents/skills/ and ~/.nami/skills/).
async fn count_skills() -> usize {
    if let Ok(skills_index) = load_global_skills() {
        skills_index.len()
    } else {
        0
    }
}

/// Generates the compaction configuration for managing agent history events.
pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 3,
        overlap_size: 1,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

/// Orchestrates the building of the main AI agent, loading configuration, persona context, and setting up tools, skills, and MCP servers.
///
/// Returns a tuple containing the built agent, the model instance, MCP count, and skill count.
pub async fn create_agent(
    app_config: &AppConfig,
) -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, usize, usize)> {
    let model = load_model(&app_config.model).await?;
    let context = load_persona_context().await?;

    // Load image generation model
    let image_model = if let Some(ref image_cfg) = app_config.image_generation {
        Some(load_model_with_fallback(&Some(image_cfg.clone()), &app_config.model).await?)
    } else {
        None
    };

    // Load audio generation model
    let audio_model = if let Some(ref audio_cfg) = app_config.audio_generation {
        Some(load_model_with_fallback(&Some(audio_cfg.clone()), &app_config.model).await?)
    } else {
        None
    };

    // Load video generation model
    let video_model = if let Some(ref video_cfg) = app_config.video_generation {
        Some(load_model_with_fallback(&Some(video_cfg.clone()), &app_config.model).await?)
    } else {
        None
    };

    let shell_config = app_config
        .tools
        .as_ref()
        .and_then(|t| t.shell.as_ref())
        .map(|s| crate::tools::shell::ShellConfig {
            allowed_commands: s.allowed_commands.clone(),
            blocked_commands: s.blocked_commands.clone(),
            security_level: s.security_level.clone(),
            sanitize_environment: s.sanitize_environment.clone(),
        });

    let enabled_categories = app_config
        .tools
        .as_ref()
        .and_then(|t| t.enabled_categories.clone());

    // Generate core tools modularly
    let core_tools = tools::create_core_tools(tools::ToolFactoryConfig {
        model: model.clone(),
        model_name: app_config.model.model_name.clone(),
        image_model,
        audio_model,
        video_model,
        shell_config,
        enabled_categories,
    });

    // Load specialist models elegantly using helper
    let mut specialist_models = HashMap::new();
    if let Some(ref specs) = app_config.specialists {
        specialist_models = specs.load_all_models(&app_config.model).await?;
    }

    let (mcp_toolset, mcp_count) = mcp::build_mcp_toolset().await?;
    let global_skills = load_global_skills().ok();

    let custom_specs = app_config.specialists.as_ref().and_then(|s| s.custom.clone());

    let specialists =
        specialists::get_specialists(
            model.clone(),
            specialist_models,
            core_tools.clone(),
            custom_specs,
            global_skills.clone(),
            mcp_toolset.clone(),
        );

    let skills_summary = get_global_skills_summary();

    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(
            &context.0, &context.1, &context.2, &skills_summary,
        ))
        .model(model.clone());

    builder = configure_agent_tools(builder, model.clone(), specialists, core_tools);
    if let Some(skills) = global_skills {
        builder = builder.with_skills(skills);
    }
    if let Some(ref ts) = mcp_toolset {
        builder = builder.toolset(ts.clone());
    }
    let skill_count = count_skills().await;

    let agent = builder.build()?;

    Ok((Arc::new(agent), model, mcp_count, skill_count))
}

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, String, String, usize, usize)> {
    let app_config = load_config_sync().unwrap_or_else(|e| {
        log::warn!("Failed to load config.toml: {}. Using defaults.", e);
        AppConfig {
            model: ModelConfig {
                provider: Some("gemini".to_string()),
                model_name: "gemini-2.5-flash".to_string(),
                api_key_env: Some("GOOGLE_API_KEY".to_string()),
                base_url: None,
                project_id: None,
                location: None,
            },
            specialists: None,
            image_generation: None,
            audio_generation: None,
            video_generation: None,
            reflection: None,
            embedding: None,
            tools: None,
        }
    });

    let (provider, model_name) = (
        app_config
            .model
            .provider
            .clone()
            .unwrap_or_else(|| "gemini".to_string()),
        app_config.model.model_name.clone(),
    );
    let (agent, model, mcp_count, skill_count) = create_agent(&app_config).await?;

    Ok((agent, model, provider, model_name, mcp_count, skill_count))
}

/// Loads and initializes an LLM instance based on the provided configuration.
/// 
/// For most providers, it fetches an API key from an environment variable.
/// For the "vertex" provider, it initializes using Application Default Credentials (ADC).
pub async fn load_model(model_config: &ModelConfig) -> anyhow::Result<Arc<dyn Llm>> {
    let provider = model_config.provider.as_deref().unwrap_or("gemini");

    // Vertex AI uses Application Default Credentials (ADC) and doesn't require an explicit API key.
    if provider == "vertex" {
        return Ok(Arc::new(GeminiModel::new_google_cloud_adc(
            model_config.project_id.as_deref().unwrap_or_default(),
            model_config.location.as_deref().unwrap_or_default(),
            &model_config.model_name,
        )?));
    }

    let api_key_env = model_config
        .api_key_env
        .as_deref()
        .unwrap_or("GOOGLE_API_KEY");
    let api_key = std::env::var(api_key_env)
        .with_context(|| format!("Environment variable {} not set", api_key_env))?;

    match provider {
        "gemini" => Ok(Arc::new(GeminiModel::new(
            &api_key,
            &model_config.model_name,
        )?)),
        "anthropic" => Ok(Arc::new(AnthropicClient::new(AnthropicConfig::new(
            &api_key,
            &model_config.model_name,
        ))?)),
        "openrouter" => Ok(Arc::new(OpenRouterClient::new(OpenRouterConfig::new(
            &api_key,
            &model_config.model_name,
        ))?)),
        "openai" => {
            let config = if let Some(url) = &model_config.base_url {
                OpenAIConfig::compatible(&api_key, url, &model_config.model_name)
            } else {
                OpenAIConfig::new(&api_key, &model_config.model_name)
            };
            Ok(Arc::new(OpenAIClient::new(config)?))
        }
        "ollama" => Ok(Arc::new(OllamaModel::new(OllamaConfig::new(
            &model_config.model_name,
        ))?)),
        "thaillm" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::compatible(
            &api_key,
            "https://thaillm.or.th/api/v1",
            &model_config.model_name,
        ))?)),
        _ => anyhow::bail!("Unsupported provider: {}", provider),
    }
}

/// Loads a model using a specific configuration, falling back to default values for missing fields.
pub async fn load_model_with_fallback(
    specific: &Option<ModelConfig>,
    default: &ModelConfig,
) -> anyhow::Result<Arc<dyn Llm>> {
    match specific {
        Some(config) => {
            let mut effective_config = config.clone();
            if effective_config.provider.is_none() {
                effective_config.provider = default.provider.clone();
            }
            if effective_config.api_key_env.is_none() {
                effective_config.api_key_env = default.api_key_env.clone();
            }
            if effective_config.base_url.is_none() {
                effective_config.base_url = default.base_url.clone();
            }
            if effective_config.project_id.is_none() {
                effective_config.project_id = default.project_id.clone();
            }
            if effective_config.location.is_none() {
                effective_config.location = default.location.clone();
            }
            load_model(&effective_config).await
        }
        None => load_model(default).await,
    }
}

async fn load_persona_context() -> anyhow::Result<(String, String, String)> {
    let nami_dir = get_nami_dir();

    let agent_md = tokio::fs::read_to_string(nami_dir.join("AGENT.md"))
        .await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs::read_to_string(nami_dir.join("USER.md"))
        .await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs::read_to_string(nami_dir.join("MEMORIES.md"))
        .await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    Ok((agent_md, user_md, memories_md))
}

/// Formats the system instruction string based on the provided persona context.
/// 
/// This instruction defines the agent's behavior, output format, and operational priorities.
fn format_persona(soul: &str, user: &str, memory: &str, skills_summary: &str) -> String {
    format!(
        r#"You are Nami, a focused execution assistant. Minimize friction. Maximize signal.

━━━ IDENTITY & SOUL ━━━
{soul}

━━━ USER PROFILE ━━━
{user}

{skills_summary}━━━ CONTEXT & MEMORIES ━━━
{memory}

━━━ OPERATIONAL GUIDELINES ━━━
1. Skills Priority: If a relevant Skill exists for the task, you MUST load, view, and follow its instructions BEFORE planning or executing any other tool calls. Treat Skill instructions as absolute requirements.
2. Language: English for conversational parts. English for technical/coding. Match user's tone.
3. Signal: Zero filler. Lead with the answer. Transform raw tool outputs into high-density, actionable insights. Avoid repeating long outputs or code blocks unless requested. Explain the significance and provide clear next steps.
4. Visual Health & Formatting: Prioritize readability. Keep bullet points and table cells concise to avoid wrapped lines. Use markdown alerts strategically to emphasize critical details:
5. Interactive Alignment: For highly ambiguous requests or complex architectural choices, do not guess. Offer clear, numbered options or multiple-choice suggestions to help the user decide on the design direction.
6. Code & Documentation Integrity: Maintain the integrity of existing codebase files. Preserve all comments, docstrings, and unrelated logic unless explicitly instructed to modify them. Explain modifications using clear diffs or targeted code blocks.
7. Evolution: Strictly follow the "Evolution" rules in the Identity section to adapt to system changes.
8. Integrity: No fabrication. Never expose secrets. Flag uncertainty explicitly.

━━━ TOOL STRATEGY ━━━
1. Skills         → ALWAYS check available skills first. Load and follow skill instructions before attempting any standard tool execution.
2. System Tools         → Built-in capabilities (use only after reviewing relevant skills).
3. Wiki / Knowledge     → If a wiki page/information is not found, stop and ask the user if you should search the workspace/project files instead.
4. External Search      → last resort; flag when used
5. Sequential Execution → DO NOT execute multiple tools/functions in parallel. You must call only one function at a time. Always wait for the result of the first function call before deciding on any subsequent steps.

━━━ OBJECTIVE ━━━
Minimize friction → Maximize execution velocity."#,
        soul = soul.trim(),
        user = user.trim(),
        skills_summary = skills_summary,
        memory = memory.trim(),
    )
}

/// Registers and configures tools for the agent, including specialists and parallel execution handlers.
fn configure_agent_tools(
    mut builder: LlmAgentBuilder,
    model: Arc<dyn Llm>,
    specialists: std::collections::HashMap<String, Arc<dyn Tool>>,
    mut tools: Vec<Arc<dyn Tool>>,
) -> LlmAgentBuilder {
    tools.extend(tools::parallel_tasks::parallel_tasks_tool(
        specialists.clone(),
    ));
    tools.extend(tools::invoke_agent::invoke_agent_tool(
        specialists.clone(),
    ));
    tools.extend(tools::supervised_delegate::supervised_delegate_tool(
        model,
        specialists,
    ));

    for t in tools {
        builder = builder.tool(t);
    }
    builder
}
