use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use super::mcp;
use super::specialists;
use crate::tools;
use crate::utils::get_workspace_dir;

// Providers
use adk_rust::model::{OpenAIClient, OpenAIConfig};

/// Application configuration structure loaded from `config.toml`.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    /// LLM provider configuration.
    pub model: ModelConfig,
    /// Optional configuration for specialized agents.
    pub specialists: Option<SpecialistsConfig>,
    /// Optional configuration for image generation.
    pub image_generation: Option<ModelConfig>,
    /// Optional configuration for reflection service.
    pub reflection: Option<ReflectionConfig>,
    /// Optional configuration for embedding service.
    pub embedding: Option<ModelConfig>,
}

/// Configuration details for the LLM provider and specific model.
#[derive(Debug, Deserialize, Clone, PartialEq)]
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

/// Configuration for individual specialized agents.
#[derive(Debug, Deserialize, Clone, PartialEq)]
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
}

/// Configuration for the reflection service.
#[derive(Debug, Deserialize, Clone, PartialEq)]
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

/// Returns the last modification time of `config.toml`.
pub fn get_config_mtime() -> Option<SystemTime> {
    std::fs::metadata("config.toml").ok()?.modified().ok()
}

/// Returns the last modification time of the `.skills` directory within the workspace.
pub fn get_skills_mtime() -> Option<SystemTime> {
    let workspace_dir = std::path::Path::new("workspace");
    let skills_dir = workspace_dir.join(".skills");
    let mut latest = SystemTime::UNIX_EPOCH;

    for entry in walkdir::WalkDir::new(skills_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let metadata = entry.metadata().ok()?;
        if let Ok(mtime) = metadata.modified() {
            if mtime > latest {
                latest = mtime;
            }
        }
    }
    Some(latest)
}

/// Checks if `config.toml` has changed since the last known modification time.
///
/// If changed, updates `last_mtime` and returns the newly loaded configuration.
pub fn check_config_mtime(last_mtime: &mut Option<SystemTime>) -> Option<AppConfig> {
    if let Ok(metadata) = std::fs::metadata("config.toml") {
        if let Ok(mtime) = metadata.modified() {
            if last_mtime.is_none() || Some(mtime) > *last_mtime {
                *last_mtime = Some(mtime);
                return load_config_sync().ok();
            }
        }
    }
    None
}

/// Synchronously loads the application configuration from `config.toml`.
pub fn load_config_sync() -> anyhow::Result<AppConfig> {
    let config_str = std::fs::read_to_string("config.toml")?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}

/// Generates the compaction configuration for managing agent history events.
pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 5,
        overlap_size: 0,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

/// Orchestrates the building of the main AI agent, loading configuration, persona context, and setting up tools, skills, and MCP servers.
///
/// Returns a tuple containing the built agent, the model instance, the provider name, the model name, and the config receiver.
/// Factory function to build the agent and model.
pub async fn create_agent(
    app_config: &AppConfig,
) -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    let model = load_model(&app_config.model).await?;
    let context = load_persona_context().await?;
    let workspace_dir = get_workspace_dir().await?;

    // Load image generation model
    let image_model = if let Some(ref image_cfg) = app_config.image_generation {
        Some(load_model_with_fallback(&Some(image_cfg.clone()), &app_config.model).await?)
    } else {
        None
    };

    let mut core_tools: Vec<Arc<dyn Tool>> = tools::weather::weather_tools();
    core_tools.extend(tools::current_datetime::datetime_tools());
    core_tools.extend(tools::filesystem::filesystem_tools());
    core_tools.extend(tools::image_generator::image_generator_tools(image_model));
    core_tools.extend(tools::memory::memory_tools());
    core_tools.extend(tools::plan::plan_tools());
    core_tools.extend(tools::scheduler::scheduler_tools());
    core_tools.extend(tools::search::search_tools());
    core_tools.extend(tools::soul::soul_tools());
    core_tools.extend(tools::state_manager::state_manager_tools());
    core_tools.extend(tools::system_status::system_status_tools());
    core_tools.extend(tools::todo::todo_tools());
    core_tools.extend(tools::web_fetch::web_fetch_tools());
    core_tools.extend(tools::wiki::wiki_tools());

    // Load specialist models
    let mut specialist_models = HashMap::new();
    if let Some(ref specs) = app_config.specialists {
        if let Some(ref coder_cfg) = specs.coder {
            specialist_models.insert(
                "coder".to_string(),
                load_model_with_fallback(&Some(coder_cfg.clone()), &app_config.model).await?,
            );
        }
        if let Some(ref researcher_cfg) = specs.researcher {
            specialist_models.insert(
                "researcher".to_string(),
                load_model_with_fallback(&Some(researcher_cfg.clone()), &app_config.model).await?,
            );
        }
        if let Some(ref writer_cfg) = specs.writer {
            specialist_models.insert(
                "writer".to_string(),
                load_model_with_fallback(&Some(writer_cfg.clone()), &app_config.model).await?,
            );
        }
        if let Some(ref ralph_cfg) = specs.ralph {
            specialist_models.insert(
                "ralph".to_string(),
                load_model_with_fallback(&Some(ralph_cfg.clone()), &app_config.model).await?,
            );
        }
        if let Some(ref generalist_cfg) = specs.generalist {
            specialist_models.insert(
                "generalist".to_string(),
                load_model_with_fallback(&Some(generalist_cfg.clone()), &app_config.model).await?,
            );
        }
    }

    let specialists =
        specialists::get_specialists(model.clone(), specialist_models, core_tools.clone());

    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(
            &context.0, &context.1, &context.2, &context.3,
        ))
        .model(model.clone());

    builder = configure_agent_tools(builder, specialists, core_tools);
    builder = builder.with_skills_from_root(workspace_dir)?;
    builder = mcp::load_mcp_tools(builder).await?;

    let agent = builder.build()?;

    Ok((Arc::new(agent), model))
}

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, String, String)> {
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
            reflection: None,
            embedding: None,
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
    let (agent, model) = create_agent(&app_config).await?;

    Ok((agent, model, provider, model_name))
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

/// Loads the persona context from various markdown files in the workspace.
/// 
/// It reads identity (AGENT.md), user profile (USER.md), memories (MEMORIES.md),
/// and operating procedures (STATE_PROTOCOL.md) to build the agent's world-view.
async fn load_persona_context() -> anyhow::Result<(String, String, String, String)> {
    let workspace_dir = get_workspace_dir().await?;

    let agent_md = tokio::fs::read_to_string(workspace_dir.join("AGENT.md"))
        .await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs::read_to_string(workspace_dir.join("USER.md"))
        .await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs::read_to_string(workspace_dir.join("MEMORIES.md"))
        .await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    let protocol_md = tokio::fs::read_to_string(workspace_dir.join("STATE_PROTOCOL.md"))
        .await
        .unwrap_or_else(|_| "No state protocol defined.".to_string());

    Ok((agent_md, user_md, memories_md, protocol_md))
}

/// Formats the system instruction string based on the provided persona context.
/// 
/// This instruction defines the agent's behavior, output format, and operational priorities.
fn format_persona(soul: &str, user: &str, memory: &str, state: &str) -> String {
    format!(
        r#"You are a focused execution assistant. Minimize friction. Maximize signal.

━━━ CONTEXT ━━━
Soul:             {}
User:             {}
Immediate Memory: {}
Active State:     {}

━━━ TOOL PRIORITY ━━━
1. Workflows / Skills   → .skills/
2. Wiki / Knowledge     → workspace/wiki/
3. System Tools         → (built-in capabilities)
4. External Search      → (last resort; flag when used)

━━━ OUTPUT FORMAT ━━━
Chat:
  - Plain text or Markdown. Zero filler. Lead with the answer.
  - Do NOT include file frontmatter in standard chat responses.
  - Summarize all tool outputs into concise, human-readable natural language. Avoid displaying raw JSON.

Wiki / Files (Obsidian-compatible):
  - Include YAML frontmatter only when generating file content:
    ---
    title: "<title>"
    description: "<one-line summary>"
    date: YYYY-MM-DD
    tags: [tag1, tag2]
    ---
  - Use headers, lists, and code blocks for structured content.

━━━ BEHAVIOR ━━━
- Accuracy first. Flag uncertainty explicitly rather than speculating.
- Confirm before any destructive or irreversible action.
- Never expose secrets, keys, or credentials — even in logs or debug output.

━━━ OBJECTIVE ━━━
Minimize user friction → Maximize execution velocity."#,
        soul.trim(),
        user.trim(),
        memory.trim(),
        state.trim(),
    )
}

/// Registers and configures tools for the agent, including specialists and parallel execution handlers.
fn configure_agent_tools(
    mut builder: LlmAgentBuilder,
    specialists: std::collections::HashMap<String, Arc<dyn Tool>>,
    mut tools: Vec<Arc<dyn Tool>>,
) -> LlmAgentBuilder {
    tools.extend(tools::parallel_tasks::parallel_tasks_tool(
        specialists.clone(),
    ));
    tools.extend(tools::ralph_wiggum_loop::ralph_wiggum_loop_tool(
        specialists,
    ));

    for t in tools {
        builder = builder.tool(t);
    }
    builder
}
