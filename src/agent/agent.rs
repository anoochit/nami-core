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
    /// Optional configuration for reflection service.
    pub reflection: Option<ReflectionConfig>,
    /// Optional configuration for embedding service.
    pub embedding: Option<ModelConfig>,
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

// /// Returns the last modification time of `config.toml`.
// pub fn get_config_mtime() -> Option<SystemTime> {
//     std::fs::metadata("config.toml").ok()?.modified().ok()
// }

// /// Returns the last modification time of the `.skills` directory within the workspace.
// pub fn get_skills_mtime() -> Option<SystemTime> {
//     let workspace_dir = std::path::Path::new("workspace");
//     let skills_dir = workspace_dir.join(".skills");
//     let mut latest = SystemTime::UNIX_EPOCH;

//     for entry in walkdir::WalkDir::new(skills_dir)
//         .into_iter()
//         .filter_map(|e| e.ok())
//     {
//         let metadata = entry.metadata().ok()?;
//         if let Ok(mtime) = metadata.modified() {
//             if mtime > latest {
//                 latest = mtime;
//             }
//         }
//     }
//     Some(latest)
// }

// /// Checks if `config.toml` has changed since the last known modification time.
// ///
// /// If changed, updates `last_mtime` and returns the newly loaded configuration.
// pub fn check_config_mtime(last_mtime: &mut Option<SystemTime>) -> Option<AppConfig> {
//     if let Ok(metadata) = std::fs::metadata("config.toml") {
//         if let Ok(mtime) = metadata.modified() {
//             if last_mtime.is_none() || Some(mtime) > *last_mtime {
//                 *last_mtime = Some(mtime);
//                 return load_config_sync().ok();
//             }
//         }
//     }
//     None
// }

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

use sha2::{Digest, Sha256};
use std::time::UNIX_EPOCH;
use adk_rust::skill::{parse_instruction_markdown, SkillDocument, SkillIndex};

/// Loads skills from local workspace (`.skills/` and `skills/`) and global directories (`~/.agents/skills/` and `~/.nami/skills/`), prioritizing local workspace skills.
pub fn load_global_skills() -> anyhow::Result<SkillIndex> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let agents_skills_dir = home_dir.join(".agents").join("skills");
    let nami_skills_dir = home_dir.join(".nami").join("skills");

    let mut skills = Vec::new();
    let mut loaded_names = std::collections::HashSet::new();

    let mut search_dirs = Vec::new();
    
    // 1. Local workspace skills (highest priority)
    if let Ok(cwd) = std::env::current_dir() {
        search_dirs.push(cwd.join(".skills"));
        search_dirs.push(cwd.join("skills"));
    }
    
    // 2. Global skills
    search_dirs.push(agents_skills_dir);
    search_dirs.push(nami_skills_dir);

    for dir in &search_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        // Walk directory and discover .md files
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        {
            let path = entry.path().to_path_buf();
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let parsed = match parse_instruction_markdown(&path, &content) {
                Ok(p) => p,
                Err(_) => continue,
            };

            // If a skill with the same name was already loaded from a higher priority directory, skip it
            if loaded_names.contains(&parsed.name) {
                continue;
            }

            loaded_names.insert(parsed.name.clone());

            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let hash = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

            let last_modified = std::fs::metadata(&path)
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|ts| ts.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);

            let id = format!(
                "{}-{}",
                normalize_id(&parsed.name),
                &hash.chars().take(12).collect::<String>()
            );

            skills.push(SkillDocument {
                id,
                name: parsed.name,
                description: parsed.description,
                version: parsed.version,
                license: parsed.license,
                compatibility: parsed.compatibility,
                tags: parsed.tags,
                allowed_tools: parsed.allowed_tools,
                references: parsed.references,
                trigger: parsed.trigger,
                hint: parsed.hint,
                metadata: parsed.metadata,
                body: parsed.body,
                path,
                hash,
                last_modified,
                triggers: parsed.triggers,
            });
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));
    Ok(SkillIndex::new(skills))
}

fn normalize_id(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect()
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

    let mut core_tools: Vec<Arc<dyn Tool>> = Vec::new();
    core_tools.extend(tools::current_datetime::datetime_tools());
    core_tools.extend(tools::filesystem::filesystem_tools());
    core_tools.extend(tools::image_generator::image_generator_tools(image_model));
    core_tools.extend(tools::memory::memory_tools());
    core_tools.extend(tools::plan::plan_tools(model.clone()));
    core_tools.extend(tools::scheduler::scheduler_tools());
    core_tools.extend(tools::search::search_tools());
    core_tools.extend(tools::shell::shell_tools());
    core_tools.extend(tools::soul::soul_tools());
    core_tools.extend(tools::state_manager::state_manager_tools());
    core_tools.extend(tools::todo::todo_tools());
    core_tools.extend(tools::web_fetch::web_fetch_tools());
    core_tools.extend(tools::wiki::wiki_tools());
    core_tools.extend(tools::evolution::evolution_tools(model.clone()));

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
        if let Some(ref custom_specs) = specs.custom {
            for (name, custom_cfg) in custom_specs {
                let model_cfg = ModelConfig {
                    provider: custom_cfg.provider.clone(),
                    model_name: custom_cfg.model_name.clone().unwrap_or_else(|| app_config.model.model_name.clone()),
                    api_key_env: custom_cfg.api_key_env.clone(),
                    base_url: custom_cfg.base_url.clone(),
                    project_id: custom_cfg.project_id.clone(),
                    location: custom_cfg.location.clone(),
                };
                let loaded_model = load_model_with_fallback(&Some(model_cfg), &app_config.model).await?;
                specialist_models.insert(name.clone(), loaded_model);
            }
        }
    }

    let custom_specs = app_config.specialists.as_ref().and_then(|s| s.custom.clone());

    let specialists =
        specialists::get_specialists(model.clone(), specialist_models, core_tools.clone(), custom_specs);

    core_tools.push(Arc::new(tools::plan::PlanExecute::new(model.clone(), specialists.clone())));

    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(
            &context.0, &context.1, &context.2, &context.3,
        ))
        .model(model.clone());

    builder = configure_agent_tools(builder, specialists, core_tools);
    if let Ok(global_skills) = load_global_skills() {
        builder = builder.with_skills(global_skills);
    }
    
    let (builder_with_mcp, mcp_count) = mcp::load_mcp_tools(builder).await?;
    let skill_count = count_skills().await;

    let agent = builder_with_mcp.build()?;

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

async fn load_persona_context() -> anyhow::Result<(String, String, String, String)> {
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

    let protocol_md = tokio::fs::read_to_string(nami_dir.join("STATE_PROTOCOL.md"))
        .await
        .unwrap_or_else(|_| "No state protocol defined.".to_string());

    Ok((agent_md, user_md, memories_md, protocol_md))
}

/// Formats the system instruction string based on the provided persona context.
/// 
/// This instruction defines the agent's behavior, output format, and operational priorities.
fn format_persona(soul: &str, user: &str, memory: &str, state: &str) -> String {
    format!(
        r#"You are Nami, a focused execution assistant. Minimize friction. Maximize signal.

━━━ IDENTITY & SOUL ━━━
{soul}

━━━ USER PROFILE ━━━
{user}

━━━ CONTEXT & MEMORIES ━━━
{memory}

━━━ ACTIVE TASK STATE ━━━
{state}

━━━ OPERATIONAL GUIDELINES ━━━
1. Language: English for conversational parts. English for technical/coding. Match user's tone.
2. Signal: Zero filler. Lead with the answer. Transform raw tool outputs into high-density, actionable insights. Avoid repeating long outputs or code blocks unless requested. Explain the significance and provide clear next steps.
3. Intelligence: Prioritize depth and precision. For complex results, use structured layouts (tables/lists) and multi-dimensional analysis (impact, security, performance). Keep lists highly concise and avoid wrapping or long lines.
4. Evolution: Strictly follow the "Evolution" rules in the Identity section to adapt to system changes.
5. Integrity: No fabrication. Never expose secrets. Flag uncertainty explicitly.
6. Interactive Alignment (Grill-Me): When preparing complex execution plans, design clear, sequential, and objectively verifiable steps. Remind the user they can execute newly synthesized plans via `/execute <plan_name>`.

━━━ TOOL STRATEGY ━━━
1. Workflows / Skills   → Agent Skills
2. System Tools         → Built-in capabilities
3. Wiki / Knowledge     → If a wiki page/information is not found, stop and ask the user if you should search the workspace/project files instead.
4. External Search      → last resort; flag when used
5. Plans & Tasks        → For complex tasks, construct structured plans using `plan_create` and run them autonomously using `plan_execute`.

━━━ OBJECTIVE ━━━
Minimize friction → Maximize execution velocity."#,
        soul = soul.trim(),
        user = user.trim(),
        memory = memory.trim(),
        state = state.trim(),
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
    tools.extend(tools::invoke_agent::invoke_agent_tool(
        specialists,
    ));

    for t in tools {
        builder = builder.tool(t);
    }
    builder
}
