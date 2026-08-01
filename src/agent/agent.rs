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

use adk_rust::skill::SkillIndex;

/// Loads skills from the three configured sources in priority order:
/// `<workspace>/.agents/skills`, `~/.agents/skills`, `~/.nami/skills`.
///
/// On name collisions the first source in priority order wins, so a workspace
/// copy overrides the agent copy, which overrides the nami copy.
///
/// Each source is indexed through `load_skill_index_with_extras` with a sentinel
/// root that never exists, so the root contributes nothing (no `.skills/`,
/// `.claude/skills/`, or convention files) and only that source's Markdown
/// skill files are parsed.
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
/// Each source is indexed through `load_skill_index_with_extras` with a sentinel
/// root that never exists, so the root contributes nothing (no `.skills/`,
/// `.claude/skills/`, or convention files) and only that source's Markdown
/// skill files are parsed.
fn load_skills_from_sources(
    workspace: &std::path::Path,
    home_dir: &std::path::Path,
) -> anyhow::Result<SkillIndex> {
    let workspace_skills_dir = workspace.join(".agents").join("skills");
    let agents_skills_dir = home_dir.join(".agents").join("skills");
    let nami_skills_dir = home_dir.join(".nami").join("skills");

    let sentinel_root = home_dir.join(".nami").join(".nami-skill-scan-root");

    let sources = [workspace_skills_dir, agents_skills_dir, nami_skills_dir];

    let mut by_name: std::collections::HashMap<String, adk_rust::skill::SkillDocument> =
        std::collections::HashMap::new();
    for dir in sources {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        let index = adk_rust::skill::load_skill_index_with_extras(&sentinel_root, &[dir])?;
        for skill in index.skills() {
            by_name
                .entry(skill.name.clone())
                .or_insert_with(|| skill.clone());
        }
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

/// Counts the number of skills across the configured sources (`<workspace>/.agents/skills/`,
/// `~/.agents/skills/` and `~/.nami/skills/`).
async fn count_skills() -> usize {
    if let Ok(skills_index) = load_global_skills() {
        skills_index.len()
    } else {
        0
    }
}

fn model_context_window(model_name: &str) -> u64 {
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

pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    let window = model_context_window(&model.name());
    let interval = if window >= 500_000 { 6 } else if window >= 200_000 { 4 } else { 3 };
    let overlap = 2;
    EventsCompactionConfig {
        compaction_interval: interval,
        overlap_size: overlap,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

/// Generates a model-aware intra-invocation compaction config (token threshold guard).
pub fn get_intra_compaction_config(model_name: &str) -> IntraCompactionConfig {
    let window = model_context_window(model_name);
    IntraCompactionConfig {
        token_threshold: window * 70 / 100,
        overlap_event_count: 10,
        chars_per_token: 4,
    }
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

    let skills_summary = get_global_skills_summary();

    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(
            &soul, &user, &memory, &skills_summary,
        ))
        .model(model.clone());

    builder = configure_agent_tools(builder, model.clone(), specialists, core_tools);
    // NOTE: Skills are deliberately NOT registered with the ADK's with_skills().
    // Doing so causes the ADK to inject [skill:name] tags into user messages at runtime,
    // which LLMs consistently hallucinate as callable function calls.
    // The agent discovers skill content via filesystem tools instead.
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
• Knowledge Vault (`wiki/`): Author concepts using Open Knowledge Format (OKF v0.2) YAML frontmatter (`type: Concept|Metric|Playbook|Attested Computation`, `title`, `description`, `status: stable`, `generated: {{ by: "agent:nami", at: "..." }}`).
• Ask numbered questions when ambiguous.
• Preserve existing code intact.
• No fabrication. Flag uncertainty.
• If unknown, ask user — do not search project files unsolicited.
• External search = last resort. Disclose its use.

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
