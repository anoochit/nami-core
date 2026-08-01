pub use crate::agent::config::*;

use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_rust::IntraCompactionConfig;
use std::collections::HashMap;
use std::sync::Arc;

use super::mcp;
use super::specialists;
use crate::tools;
use crate::utils::get_nami_dir;

use adk_rust::skill::{load_skill_index_with_extras, select_skill_prompt_block, SelectionPolicy, SkillIndex};

/// Loads skills from the three configured sources in priority order:
/// `<workspace>/.agents/skills`, `~/.agents/skills`, `~/.nami/skills`.
///
/// On name collisions the first source in priority order wins, so a workspace
/// copy overrides the agent copy, which overrides the nami copy.
///
/// Each source is indexed through `load_skill_index_with_extras` using the
/// source directory itself as the root, so convention files (AGENTS.md,
/// CLAUDE.md, etc.) at the source root are discovered alongside strict
/// `.skills/` definitions.
pub fn load_global_skills() -> anyhow::Result<SkillIndex> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    let workspace = match std::env::var("NAMI_WORKSPACE") {
        Ok(ws) if !ws.is_empty() => std::path::PathBuf::from(ws),
        _ => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };

    load_skills_from_sources(&workspace, &home_dir)
}

/// Loads skills from the three configured sources under explicit workspace and
/// home roots, in priority order: `<workspace>/.agents/skills`, `~/.agents/skills`,
/// `~/.nami/skills`.
///
/// On name collisions the first source in priority order wins, so a workspace
/// copy overrides the agent copy, which overrides the nami copy.
///
/// Each source is indexed through `load_skill_index_with_extras` using the
/// parent directory as root (for convention files) and the skills directory
/// as an extra dir (for skill files in `skills/`).
fn load_skills_from_sources(
    workspace: &std::path::Path,
    home_dir: &std::path::Path,
) -> anyhow::Result<SkillIndex> {
    let sources = [
        (workspace.join(".agents"), workspace.join(".agents").join("skills")),
        (home_dir.join(".agents"), home_dir.join(".agents").join("skills")),
        (home_dir.join(".nami"), home_dir.join(".nami").join("skills")),
    ];

    let mut all_skills = Vec::new();
    for (root, skills_dir) in sources {
        if !root.exists() || !root.is_dir() {
            continue;
        }
        // Use parent as root (for convention files) and skills dir as extra (for skills/ files)
        let extra_dirs = if skills_dir.exists() && skills_dir.is_dir() {
            vec![skills_dir]
        } else {
            vec![]
        };
        let index = load_skill_index_with_extras(&root, &extra_dirs)?;
        all_skills.extend(index.skills().iter().cloned());
    }

    // Deduplicate by name, priority order wins (first source wins)
    let mut by_name: std::collections::HashMap<String, adk_rust::skill::SkillDocument> =
        std::collections::HashMap::new();
    for skill in all_skills {
        by_name.entry(skill.name.clone()).or_insert(skill);
    }

    let mut skills: Vec<_> = by_name.into_values().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));
    Ok(SkillIndex::new(skills))
}

/// Returns a generic instruction about skill guide discovery.
/// Skill names are deliberately omitted to prevent LLMs from hallucinating them as callable functions.
pub fn get_global_skills_summary() -> String {
    "Skill reference guides are available as Markdown files. When you need domain knowledge or step-by-step instructions for a task, use the filesystem tool to find and read the relevant SKILL.md file from the workspace .agents/skills/, ~/.agents/skills/ or ~/.nami/skills/ directories.".to_string()
}

/// Escapes `{` characters in skill body content so that
/// `inject_session_state` does not interpret them as template placeholders.
/// Code blocks in skill files (e.g. JavaScript template literals like `${deepLink}`)
/// would otherwise be parsed as required state variables and cause runtime errors.
fn escape_template_braces(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    for ch in body.chars() {
        match ch {
            '{' => out.push_str("{ "),
            '}' => out.push_str("} "),
            _ => out.push(ch),
        }
    }
    out
}

/// Counts the number of skills across the configured sources (`<workspace>/.agents/skills/`,
/// `~/.agents/skills/` and `~/.nami/skills/`).
async fn count_skills() -> usize {
    if let Ok(skills_index) = load_global_skills() {
        skills_index.len()
    } else {
        0
    }
}

/// Looks up model context window from config overrides, then falls back to hardcoded heuristics.
fn model_context_window(model_name: &str, overrides: &Option<HashMap<String, u64>>) -> u64 {
    // 1. Check config overrides (case-insensitive substring match)
    if let Some(windows) = overrides {
        let lower = model_name.to_lowercase();
        for (key, &size) in windows {
            if lower.contains(&key.to_lowercase()) {
                return size;
            }
        }
    }
    // 2. Fall back to hardcoded heuristics
    let n = model_name.to_lowercase();
    if n.contains("gemini-2.5") || n.contains("gemini-3") || n.contains("gemini-2.0") {
        1_000_000
    } else if n.contains("claude") {
        200_000
    } else if n.contains("gpt-4") || n.contains("gpt-3.5") {
        128_000
    } else if n.contains("deepseek") {
        64_000
    } else if n.contains("command") || n.contains("aya") {
        128_000
    } else {
        128_000
    }
}

pub fn get_compaction_config(
    model: Arc<dyn Llm>,
    compaction_cfg: &Option<crate::agent::config::CompactionConfig>,
) -> EventsCompactionConfig {
    let window = model_context_window(
        &model.name(),
        &compaction_cfg.as_ref().and_then(|c| c.model_context_windows.clone()),
    );
    let interval = compaction_cfg
        .as_ref()
        .and_then(|c| c.compaction_interval)
        .unwrap_or_else(|| {
            if window >= 500_000 { 6 } else if window >= 200_000 { 4 } else { 3 }
        });
    let overlap = compaction_cfg
        .as_ref()
        .and_then(|c| c.overlap_size)
        .unwrap_or(2) as u32;
    EventsCompactionConfig {
        compaction_interval: interval,
        overlap_size: overlap,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

/// Generates a model-aware intra-invocation compaction config (token threshold guard).
pub fn get_intra_compaction_config(
    model_name: &str,
    compaction_cfg: &Option<crate::agent::config::CompactionConfig>,
) -> IntraCompactionConfig {
    let window = model_context_window(
        model_name,
        &compaction_cfg.as_ref().and_then(|c| c.model_context_windows.clone()),
    );
    let threshold_pct = compaction_cfg
        .as_ref()
        .and_then(|c| c.token_threshold_pct)
        .unwrap_or(70);
    let overlap_event_count = compaction_cfg
        .as_ref()
        .and_then(|c| c.overlap_event_count)
        .unwrap_or(10);
    let chars_per_token = compaction_cfg
        .as_ref()
        .and_then(|c| c.chars_per_token)
        .unwrap_or(4);
    IntraCompactionConfig {
        token_threshold: window * threshold_pct / 100,
        overlap_event_count,
        chars_per_token,
    }
}

/// Builds the intra-compaction summarizer (may use a separate model).
pub fn get_intra_compaction_summarizer(
    model: Arc<dyn Llm>,
    compaction_cfg: &Option<crate::agent::config::CompactionConfig>,
) -> Arc<dyn adk_core::BaseEventsSummarizer> {
    let summarizer_llm = build_summarizer_llm_sync(model, compaction_cfg);
    Arc::new(LlmEventSummarizer::new(summarizer_llm))
}

/// Builds the summarizer LLM instance (separate model or fallback to agent model).
/// This is the synchronous version that can be called at entry points.
pub fn build_summarizer_llm_sync(
    fallback_model: Arc<dyn Llm>,
    compaction_cfg: &Option<crate::agent::config::CompactionConfig>,
) -> Arc<dyn Llm> {
    if let Some(cfg) = compaction_cfg {
        if let Some(summarizer) = &cfg.summarizer {
            let model_cfg = crate::agent::config::ModelConfig {
                provider: summarizer.provider.clone(),
                model_name: summarizer.model_name.clone(),
                api_key_env: summarizer.api_key_env.clone(),
                base_url: summarizer.base_url.clone(),
                project_id: None,
                location: None,
            };
            // Try to load the summarizer model synchronously
            match tokio::runtime::Handle::current().block_on(
                crate::agent::config::load_model(&model_cfg)
            ) {
                Ok(model) => return model,
                Err(e) => {
                    log::warn!("Failed to load summarizer model '{}', falling back to agent model: {}",
                        summarizer.model_name, e);
                }
            }
        }
    }
    fallback_model
}

/// Async version of build_summarizer_llm for use in async contexts.
pub async fn build_summarizer_llm(
    fallback_model: Arc<dyn Llm>,
    compaction_cfg: &Option<crate::agent::config::CompactionConfig>,
) -> Arc<dyn Llm> {
    if let Some(cfg) = compaction_cfg {
        if let Some(summarizer) = &cfg.summarizer {
            let model_cfg = crate::agent::config::ModelConfig {
                provider: summarizer.provider.clone(),
                model_name: summarizer.model_name.clone(),
                api_key_env: summarizer.api_key_env.clone(),
                base_url: summarizer.base_url.clone(),
                project_id: None,
                location: None,
            };
            match crate::agent::config::load_model(&model_cfg).await {
                Ok(model) => return model,
                Err(e) => {
                    log::warn!("Failed to load summarizer model '{}', falling back to agent model: {}",
                        summarizer.model_name, e);
                }
            }
        }
    }
    fallback_model
}

/// Orchestrates the building of the main AI agent, loading configuration, persona context, and setting up tools, skills, and MCP servers.
///
/// Returns a tuple containing the built agent, the model instance, MCP count, and skill count.
pub async fn create_agent(
    app_config: &AppConfig,
) -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, usize, usize)> {
    let (
        model,
        (soul, user, memory),
        image_model,
        audio_model,
        video_model,
        specialist_models,
        (mcp_toolset, mcp_count),
    ) = tokio::try_join!(
        load_model(&app_config.model),
        load_persona_context(),
        load_optional_model(&app_config.image_generation, &app_config.model),
        load_optional_model(&app_config.audio_generation, &app_config.model),
        load_optional_model(&app_config.video_generation, &app_config.model),
        async {
            match &app_config.specialists {
                Some(specs) => specs.load_all_models(&app_config.model).await,
                None => Ok(HashMap::new()),
            }
        },
        mcp::build_mcp_toolset(),
    )?;

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

    let core_tools = tools::create_core_tools(tools::ToolFactoryConfig {
        model: model.clone(),
        model_name: app_config.model.model_name.clone(),
        image_model,
        audio_model,
        video_model,
        shell_config,
        enabled_categories,
    });

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

    // Build the static persona (without skill content — skills are injected dynamically per query)
    let static_persona = format_persona(&soul, &user, &memory, &get_global_skills_summary());

    // Clone skills index for the instruction provider closure
    let skills_index_for_provider = global_skills.clone();

    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction_provider(Box::new(move |ctx| {
            let persona = static_persona.clone();
            let skills_index = skills_index_for_provider.clone();
            Box::pin(async move {
                // Extract user query text for skill scoring
                let user_query = ctx.user_content().parts.iter()
                    .filter_map(|part| match part {
                        adk_rust::Part::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Dynamically select the best-matching skill at runtime
                let skill_content = if let Some(index) = &skills_index {
                    if let Some((_, prompt_block)) = select_skill_prompt_block(
                        index,
                        &user_query,
                        &SelectionPolicy::default(),
                        2000,
                    ) {
                        // Strip [skill:name] and [/skill] tags to prevent LLM hallucination
                        let stripped = prompt_block
                            .replace("[/skill]", "");
                        // Extract just the content after the opening tag's closing bracket
                        if let Some(start) = stripped.find("]\n") {
                            let content = &stripped[start + 2..];
                            Some(escape_template_braces(content))
                        } else {
                            Some(escape_template_braces(&stripped))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Combine static persona with dynamic skill content
                match skill_content {
                    Some(content) if !content.is_empty() => {
                        Ok(format!("{persona}\n\n## Active Skill\n{content}"))
                    }
                    _ => Ok(persona),
                }
            })
        }))
        .model(model.clone());

    builder = configure_agent_tools(builder, model.clone(), specialists, core_tools);
    // NOTE: Skills are NOT registered with the ADK's with_skills() on the main agent.
    // That would inject [skill:name] tags into user messages, causing LLMs to hallucinate
    // them as callable function calls. Instead, skills are scored dynamically at runtime
    // via instruction_provider and injected into the system instruction without tags.
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
            compaction: None,
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

async fn load_persona_context() -> anyhow::Result<(String, String, String)> {
    let nami_dir = get_nami_dir();

    let (agent_md, user_md, memories_md) = tokio::join!(
        tokio::fs::read_to_string(nami_dir.join("AGENT.md")),
        tokio::fs::read_to_string(nami_dir.join("USER.md")),
        tokio::fs::read_to_string(nami_dir.join("MEMORIES.md")),
    );

    Ok((
        agent_md.unwrap_or_else(|_| "Standard Assistant".to_string()),
        user_md.unwrap_or_else(|_| "Developer".to_string()),
        memories_md.unwrap_or_else(|_| "No previous memories.".to_string()),
    ))
}

/// Formats the system instruction string based on the provided persona context.
/// 
/// This instruction defines the agent's behavior, output format, and operational priorities.
fn format_persona(soul: &str, user: &str, memory: &str, skills_summary: &str) -> String {
    format!(
        r#"You are Nami — a focused execution assistant. Drive tasks to completion with minimal friction.

— IDENTITY —
{soul}

— USER —
{user}

— SKILLS —
{skills_summary}

— MEMORY —
{memory}

— RULES —
• Default to English. Match user's tone and technical level.
• Lead with direct answers. Summarize tool output; never dump verbatim unless asked.
• Use Markdown (tables, alerts). Keep cells short.
• Knowledge Search & Retrieval: ALWAYS search in the Knowledge Base (`km/`) FIRST before resorting to external Google search (`google_search`).
• Knowledge Vault (`km/`): Author concepts using Open Knowledge Format (OKF v0.2) YAML frontmatter (`type: Concept|Metric|Playbook|Attested Computation`, `title`, `description`, `status: stable`, `generated: {{ by: "agent:nami", at: "..." }}`).
• Auto-Knowledge Capture: Whenever new knowledge/facts are retrieved from external Google searches, ALWAYS automatically add/save that new knowledge to the Knowledge Base vault (`add_km_page`) using OKF v0.2 format.
• Ask numbered questions when ambiguous.
• Preserve existing code intact.
• No fabrication. Flag uncertainty.
• If unknown, ask user — do not search project files unsolicited.
• External search = last resort after checking Knowledge Base. Disclose its use.

— GOAL —
Minimize friction → Maximize velocity"#,
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
    tools.reserve(tools.len() + 3);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_subdir(label: &str) -> PathBuf {
        let seq = TEST_SEQ.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!("nami-skill-test-{}-{seq}", label));
        let _ = std::fs::remove_dir_all(&base);
        base
    }

    fn write_skill(dir: &PathBuf, name: &str, description: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.md")),
            format!("---\nname: {name}\ndescription: {description}\n---\nbody for {name}.\n"),
        )
        .unwrap();
    }

    #[test]
    fn load_skills_from_sources_prefers_workspace_over_agent_over_nami() {
        let base = temp_subdir("merge");
        let ws = base.join("ws");
        let home = base.join("home");

        write_skill(
            &ws.join(".agents").join("skills"),
            "search",
            "workspace-search",
        );
        write_skill(
            &home.join(".agents").join("skills"),
            "search",
            "agent-search",
        );
        write_skill(&home.join(".nami").join("skills"), "search", "nami-search");
        write_skill(
            &home.join(".nami").join("skills"),
            "nami-only",
            "nami-only-desc",
        );

        let index = load_skills_from_sources(&ws, &home).unwrap();

        // Collision: workspace wins.
        let search = index.find_by_name("search").expect("search skill present");
        assert_eq!(search.description, "workspace-search");
        assert_eq!(
            index.skills().iter().filter(|s| s.name == "search").count(),
            1,
            "same-named skills across sources must be deduplicated"
        );
        // Non-colliding skill from the lowest-priority source still loads.
        assert_eq!(
            index
                .find_by_name("nami-only")
                .map(|s| s.description.as_str()),
            Some("nami-only-desc")
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_skills_from_sources_loads_convention_files() {
        let base = temp_subdir("convention");
        let ws = base.join("ws");
        let home = base.join("home");

        // Create AGENTS.md at workspace .agents root
        std::fs::create_dir_all(&ws.join(".agents")).unwrap();
        std::fs::write(
            ws.join(".agents").join("AGENTS.md"),
            "# Workspace Agent Instructions\nAlways use cargo test before commit.\n",
        )
        .unwrap();

        // Create CLAUDE.md at home .agents root
        std::fs::create_dir_all(&home.join(".agents")).unwrap();
        std::fs::write(
            home.join(".agents").join("CLAUDE.md"),
            "# Claude Instructions\nPrefer rg over grep.\n",
        )
        .unwrap();

        let index = load_skills_from_sources(&ws, &home).unwrap();

        // AGENTS.md should be loaded as "agents" skill
        let agents = index.find_by_name("agents").expect("agents skill present");
        assert_eq!(agents.description, "Workspace Agent Instructions");
        assert!(agents.tags.contains(&"agents-md".to_string()));
        assert!(agents.body.contains("cargo test before commit"));

        // CLAUDE.md should be loaded as "claude" skill
        let claude = index.find_by_name("claude").expect("claude skill present");
        assert_eq!(claude.description, "Claude Instructions");
        assert!(claude.tags.contains(&"claude-md".to_string()));
        assert!(claude.body.contains("rg over grep"));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_skills_from_sources_agent_wins_over_nami_when_no_workspace_copy() {
        let base = temp_subdir("no-ws");
        let ws = base.join("ws");
        let home = base.join("home");

        write_skill(
            &home.join(".agents").join("skills"),
            "search",
            "agent-search",
        );
        write_skill(&home.join(".nami").join("skills"), "search", "nami-search");

        let index = load_skills_from_sources(&ws, &home).unwrap();

        let search = index.find_by_name("search").expect("search skill present");
        assert_eq!(search.description, "agent-search");

        let _ = std::fs::remove_dir_all(&base);
    }
}
