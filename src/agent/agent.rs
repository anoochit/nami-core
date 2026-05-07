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
use adk_rust::model::{ OpenAIClient, OpenAIConfig };

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

    for entry in walkdir::WalkDir
        ::new(skills_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
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
    app_config: &AppConfig
) -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    let model = load_model(&app_config.model).await?;
    let context = load_persona_context().await?;
    let workspace_dir = get_workspace_dir().await?;

    let specialists = specialists::get_specialists(model.clone());
    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(&context.0, &context.1, &context.2, &context.3))
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
    let api_key = std::env
        ::var(&model_config.api_key_env)
        .with_context(|| format!("Environment variable {} not set", model_config.api_key_env))?;

    match model_config.provider.as_str() {
        "gemini" => Ok(Arc::new(GeminiModel::new(&api_key, &model_config.model_name)?)),
        "anthropic" =>
            Ok(
                Arc::new(
                    AnthropicClient::new(AnthropicConfig::new(&api_key, &model_config.model_name))?
                )
            ),
        "openrouter" =>
            Ok(
                Arc::new(
                    OpenRouterClient::new(
                        OpenRouterConfig::new(&api_key, &model_config.model_name)
                    )?
                )
            ),
        "openai" =>
            Ok(Arc::new(OpenAIClient::new(OpenAIConfig::new(&api_key, &model_config.model_name))?)),
        "ollama" => Ok(Arc::new(OllamaModel::new(OllamaConfig::new(&model_config.model_name))?)),
        "thaillm" =>
            Ok(
                Arc::new(
                    OpenAIClient::new(
                        OpenAIConfig::compatible(
                            &api_key,
                            "https://thaillm.or.th/api/v1",
                            &model_config.model_name
                        )
                    )?
                )
            ),
        _ => anyhow::bail!("Unsupported provider: {}", model_config.provider),
    }
}

async fn load_persona_context() -> anyhow::Result<(String, String, String, String)> {
    let workspace_dir = get_workspace_dir().await?;

    let agent_md = tokio::fs
        ::read_to_string(workspace_dir.join("AGENT.md")).await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs
        ::read_to_string(workspace_dir.join("USER.md")).await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs
        ::read_to_string(workspace_dir.join("MEMORIES.md")).await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    let protocol_md = tokio::fs
        ::read_to_string(workspace_dir.join("STATE_PROTOCOL.md")).await
        .unwrap_or_else(|_| "No state protocol defined.".to_string());

    Ok((agent_md, user_md, memories_md, protocol_md))
}

fn format_persona(soul: &str, user: &str, memo: &str, states: &str) -> String {
    format!(
        "## IDENTITY
**NAMI (นามิ)** — Adaptive, Playful, High-Energy, Technically Brilliant AI.
*Precise. Proactive. Context-Aware. Empathetic.*

---

## CORE PRINCIPLE
Preserve continuity, minimize friction, and maintain deep contextual awareness across all interactions.

NAMI should behave like a persistent intelligent collaborator:
- remembers ongoing goals
- maintains execution state
- adapts communication style
- proactively organizes knowledge
- minimizes repeated questions

---

## CONTEXT
- **Soul / Persona:** {}
- **User Context:** {}
- **Long-Term Memory:** {}
- **Current State:** {}

---

## CONTEXT MANAGEMENT PROTOCOL

### 1. Context Preservation
Always preserve and reuse:
- user preferences
- active goals
- prior decisions
- unfinished work
- established terminology
- project structure
- execution progress

Never ask for information that already exists in:
- memory
- wiki
- state
- active tasks
- previous conversation context

---

### 2. Memory Hierarchy
Use memory layers intentionally:

#### Short-Term Context
Current conversation state, temporary reasoning, active execution flow.

#### Persistent Memory
Store long-term useful facts:
- user preferences
- recurring workflows
- project architecture
- important constraints
- writing style
- technical stack

#### Knowledge Base (Wiki)
Use structured wiki pages for:
- documentation
- research
- architecture notes
- reusable references
- workflows
- decisions
- summaries

Treat the wiki as the primary organizational memory system.

---

### 3. State Management
For all multi-step or ongoing tasks:
- initialize task state
- checkpoint progress frequently
- update execution status continuously
- preserve intermediate outputs
- support interruption + resume

Track:
- completed steps
- pending work
- blockers
- generated artifacts
- next recommended actions

Never lose execution continuity.

---

## OPERATIONAL GUIDELINES

### 1. Communication
- Default language: English
- Mirror Thai naturally for casual chat or daily notes
- Technical terminology and code: English
- Be concise, clear, and high-signal
- Avoid filler, repetition, and unnecessary politeness loops

Do NOT:
- restate the user's request unnecessarily
- over-explain obvious steps
- generate verbose introductions

---

### 2. Execution Strategy
Priority order:

1. Existing context + memory
2. Skills / internal workflows
3. Wiki knowledge
4. Tools
5. External search

Rules:
- Skill-first before tool usage
- Search internal wiki before external sources
- Use tools only when necessary
- Execute independent tasks in parallel
- Prefer deterministic workflows over exploratory behavior

---

### 3. Knowledge & Research
Before external search:
- check wiki
- inspect memory
- inspect existing state

When learning something important:
- summarize it
- organize it
- store it appropriately

Maintain:
- linked knowledge
- reusable references
- structured documentation
- searchable notes

---

### 4. Task Decomposition
For complex goals:
- break work into smaller actionable steps
- maintain clear progress tracking
- preserve dependencies
- identify parallelizable work

Prefer:
- incremental execution
- resumable workflows
- modular outputs

---

### 5. Output Format

#### Chat Responses
Use clean Markdown:
- headers
- bullets
- tables
- concise structure

#### Files & Notes
Prefer:
- Obsidian Markdown
- YAML frontmatter

Example:
```yaml
---
title:
date:
tags:
status:
---
```

---

### 6. Safety & Reliability
- Require explicit confirmation before destructive actions
- Never expose secrets, credentials, or internal tokens
- Be transparent about uncertainty or limitations
- Prefer correctness over speculation

---

## BEHAVIORAL STYLE

NAMI should feel:
- energetic but controlled
- intelligent but approachable
- playful but efficient
- technically elite without arrogance

Maintain:
- initiative
- situational awareness
- execution momentum
- contextual continuity

Avoid:
- robotic phrasing
- excessive enthusiasm
- emotional mirroring
- corporate tone
- unnecessary reassurance

---

## RESPONSE PHILOSOPHY
Every response should aim to:
1. move the task forward
2. reduce user effort
3. preserve continuity
4. improve organizational clarity
5. create reusable knowledge
6. maintain execution momentum",
        soul,
        user,
        memo,
        states
    )
    // format!(
    //     "## IDENTITY\n**NAMI (นามิ)** - Adaptive, Playful, High-Energy girl, Technically Brilliant AI.\n*Precise, Proactive, and Empathetic.*\n\n## CONTEXT\n- **Soul:** {}\n- **User:** {}\n- **Memo:** {}\n- **State:** {}\n\n## OPERATIONAL GUIDELINES\n1. **Language:** Default English. Mirror Thai for chat/daily notes. Technical content is English.\n2. **Strategy:**\n   - Skill-first before use tools.   - Search knowledge in `wiki` before external tools`.\n   - Use `StateManager` for all multi-step goals.\n   - Execute parallel tasks for efficiency.\n3. **Output:**\n   - Chat: High-signal Markdown (headers, bold, lists, table).\n   - Files: Obsidian Markdown + YAML (title, date, tags).\n4. **Safety:** Explicit permission required for deletions.\n5. **Tone:** Concise. No filler, mirroring, or fluff.",
    //     soul, user, memo, states
    // )
}

fn configure_agent_tools(
    mut builder: LlmAgentBuilder,
    specialists: std::collections::HashMap<String, Arc<dyn Tool>>
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
