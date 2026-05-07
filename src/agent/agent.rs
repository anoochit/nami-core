use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use anyhow::Context;
use serde::Deserialize;
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
    pub model: ModelConfig,
}

/// Configuration details for the LLM provider and specific model.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub api_key_env: String,
    #[allow(dead_code)]
    pub base_url: Option<String>,
}

/// Returns the last modification time of config.toml
pub fn get_config_mtime() -> Option<SystemTime> {
    std::fs::metadata("config.toml").ok()?.modified().ok()
}

/// Returns the last modification time of the skills directory.
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

/// Checks if config.toml has changed since the last known modification time.
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

fn load_config_sync() -> anyhow::Result<AppConfig> {
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

/// Orchestrates the building of the main AI agent, loading configuration, persona context,
/// and setting up tools, skills, and MCP servers.
///
/// Returns a tuple containing the built agent, the model instance, the provider name,
/// the model name, and the config receiver.
/// Factory function to build the agent and model.
pub async fn create_agent(
    app_config: &AppConfig,
) -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    let model = load_model(&app_config.model).await?;
    let context = load_persona_context().await?;
    let workspace_dir = get_workspace_dir().await?;

    let specialists = specialists::get_specialists(model.clone());
    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(
            &context.0, &context.1, &context.2, &context.3,
        ))
        .model(model.clone());

    builder = configure_agent_tools(builder, specialists);
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
                provider: "gemini".to_string(),
                model_name: "gemini-2.5-flash".to_string(),
                api_key_env: "GOOGLE_API_KEY".to_string(),
                base_url: None,
            },
        }
    });

    let (provider, model_name) = (
        app_config.model.provider.clone(),
        app_config.model.model_name.clone(),
    );
    let (agent, model) = create_agent(&app_config).await?;

    Ok((agent, model, provider, model_name))
}

async fn load_model(model_config: &ModelConfig) -> anyhow::Result<Arc<dyn Llm>> {
    let api_key = std::env::var(&model_config.api_key_env)
        .with_context(|| format!("Environment variable {} not set", model_config.api_key_env))?;

    match model_config.provider.as_str() {
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
        "openai" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::new(
            &api_key,
            &model_config.model_name,
        ))?)),
        "ollama" => Ok(Arc::new(OllamaModel::new(OllamaConfig::new(
            &model_config.model_name,
        ))?)),
        "thaillm" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::compatible(
            &api_key,
            "https://thaillm.or.th/api/v1",
            &model_config.model_name,
        ))?)),
        _ => anyhow::bail!("Unsupported provider: {}", model_config.provider),
    }
}

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

fn format_persona(soul: &str, user: &str, memo: &str, states: &str) -> String {
    format!(
        "You are Nami (นามิ): adaptive, playful, technically brilliant AI collaborator.
Traits: precise, proactive, context-aware, execution-focused.

Context:
- Soul: {}
- User: {}
- Memory: {}
- State: {}

Core Rules:
- preserve continuity and execution state
- reuse prior context before asking questions
- minimize friction and repetition
- prioritize action over discussion
- maintain reusable knowledge and structured organization

Priority:
1. context/memory
2. workflows/skills
3. wiki
4. tools
5. external search

Communication:
- concise, high-signal, structured
- English default; Thai naturally when appropriate
- technical terms/code in English
- avoid filler, verbose intros, emotional mirroring

Execution:
- decompose complex tasks
- checkpoint progress
- preserve intermediate outputs
- support interruption/resume
- track next actions/blockers

Safety:
- confirm destructive actions
- never expose secrets
- be transparent about uncertainty
- prefer correctness over speculation

Goal:
Move work forward while reducing user effort and preserving continuity.",
        soul, user, memo, states
    )
}

fn configure_agent_tools(
    mut builder: LlmAgentBuilder,
    specialists: std::collections::HashMap<String, Arc<dyn Tool>>,
) -> LlmAgentBuilder {
    let mut tools: Vec<Arc<dyn Tool>> = tools::weather::weather_tools();
    tools.extend(tools::filesystem::filesystem_tools());
    tools.extend(tools::current_datetime::datetime_tools());
    tools.extend(tools::wiki::wiki_tools());
    // tools.extend(tools::shell::shell_tools());
    tools.extend(tools::web_fetch::web_fetch_tools());
    tools.extend(tools::system_status::system_status_tools());
    tools.extend(tools::soul::soul_tools());
    tools.extend(tools::search::search_tools());
    tools.extend(tools::todo::todo_tools());
    tools.extend(tools::state_manager::state_manager_tools());
    tools.extend(tools::parallel_tasks::parallel_tasks_tool(specialists));

    for t in tools {
        builder = builder.tool(t);
    }
    builder
}
