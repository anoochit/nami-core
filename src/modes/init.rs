use adk_session::SqliteSessionService;
use inquire::{Password, Select, Text};
use std::fs::File;
use std::io::Write;
use termimad::{MadSkin, mad_print_inline};

fn write_file(name: &str, content: &str) -> std::io::Result<()> {
    let nami_dir = crate::utils::get_nami_dir();
    let dest_path = nami_dir.join(name);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}


pub async fn run_init() -> anyhow::Result<()> {
    let skin = MadSkin::default();

    skin.print_text("# AI Agent Initializer\n");

    // --- 1. LLM Configuration ---
    skin.print_text("### 1. LLM Configuration\n");
    let providers = vec![
        "anthropic",
        "gemini",
        "vertex",
        "ollama",
        "openai",
        "openrouter",
        "thaillm",
        "custom",
    ];
    let provider_selection = Select::new("Choose LLM Provider:", providers).prompt()?;

    let provider = if provider_selection == "custom" {
        Text::new("Enter Custom Provider:").prompt()?
    } else {
        provider_selection.to_string()
    };

    let models = match provider.as_str() {
        "anthropic" => vec![
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-haiku-4-5-20251001",
            "claude-opus-4-5-20251101",
            "claude-sonnet-4-5-20250929",
            "custom",
        ],
        "gemini" => vec![
            "gemini-pro-latest",
            "gemini-flash-latest",
            "gemini-3.1-pro-preview",
            "gemini-3-flash-preview",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "custom",
        ],
        "vertex" => vec![
            "gemini-3.1-pro-preview",
            "gemini-3-flash-preview",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "custom",
        ],
        "ollama" => vec!["deepseek-r1:1.5b", "custom"],
        "openai" => vec!["gpt-5", "gpt-4.1", "custom"],
        "openrouter" => vec![
            "anthropic/claude-3.5-sonnet",
            "tencent/hy3-preview:free",
            "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
            "nvidia/nemotron-3-super-120b-a12b:free",
            "openrouter/free",
            "custom",
        ],
        "thaillm" => vec![
            "openthaigpt-thaillm-8b-instruct-v7.2",
            "pathumma-thaillm-qwen3-8b-think-3.0.0",
            "typhoon-s-thaillm-8b-instruct",
            "thalle-0.2-thaillm-8b-fa",
            "custom",
        ],
        _ => vec!["custom"],
    };

    let model_selection = Select::new("Choose Model Name:", models).prompt()?;

    let model_name = if model_selection == "custom" {
        Text::new("Enter Model Name:").prompt()?
    } else {
        model_selection.to_string()
    };

    let api_key = if provider != "vertex" {
        Password::new("Enter LLM API Key:")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?
    } else {
        String::new()
    };

    let (project_id, location) = if provider == "vertex" {
        let pid = Text::new("Enter Google Cloud Project ID:").prompt()?;
        let loc = Text::new("Enter Google Cloud Location (e.g., us-central1):").prompt()?;
        (Some(pid), Some(loc))
    } else {
        (None, None)
    };

    // Determine Env Var Name based on provider
    let api_key_env = match provider.as_str() {
        "anthropic" => "ANTHROPIC_API_KEY",
        "gemini" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "thaillm" => "THAILLM_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "ollama" => "OLLAMA_API_KEY",
        _ => "GOOGLE_API_KEY",
    };

    // --- 2. Search Configuration ---
    skin.print_text("\n### 2. Search Configuration\n");
    let serper_api_key = Password::new("Enter Serper API Key (optional):")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    // --- 3. Bot Configuration ---
    skin.print_text("\n### 3. Bot Configuration\n");
    let telegram_key = Password::new("Enter Telegram API Key (optional):")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    let line_secret = Password::new("Enter LINE Channel Secret (optional):")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    let line_token = Password::new("Enter LINE Channel Access Token (optional):")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    // --- 4. Observability Configuration ---
    skin.print_text("\n### 4. Observability Configuration\n");
    let otel_collector = Text::new("Enter OTEL_COLLECTOR URL (e.g., http://localhost:4317) (optional):")
        .prompt()?;

    // --- 5. Nami API Configuration ---
    skin.print_text("\n### 5. Nami API Configuration\n");
    let nami_api_key = Password::new("Enter Nami API Key (optional):")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    // --- File Generation ---

    // Ensure global directory exists
    let nami_dir = crate::utils::get_nami_dir();
    std::fs::create_dir_all(nami_dir.join("skills/cli-help"))?;

    let project_id_str = project_id.unwrap_or_default();
    let location_str = location.unwrap_or_default();
    let current_dir = std::env::current_dir()?.to_string_lossy().replace('\\', "/");

    // 1. config.toml
    let config_content = format!(
        r#"[model]
# Provider type: "anthropic","gemini","vertex", "ollama", "openai", "openrouter" or "thaillm",
provider = "{provider}"
# The specific model identifier
model_name = "{model_name}"
# The environment variable name that holds the API key
api_key_env = "{api_key_env}"

# Vertex AI settings
project_id = "{project_id_str}"
location = "{location_str}"

# Optional settings for OpenAI-compatible providers
# base_url = "https://api.openai.com/v1"

[workspaces]
# The default active workspace path when Nami is run outside any registered workspaces
active = "{current_dir}"
# List of registered workspace directories. Nami automatically detects and activates
# the correct workspace context if you run Nami inside any of these folders or their subfolders.
list = ["{current_dir}"]

[commands]
# Custom command definitions
[commands."/plan"]
template = "plan_create(name='auto', objective='{{args}}')"
help = "Create an AI research plan"

[commands."/wiki"]
template = "wiki_search: {{args}}"
help = "Search the project wiki"

[commands."/memo"]
template = "add_memory: {{args}}"
help = "Save information to memory"

[commands."/parallel"]
template = "Execute the following tasks in parallel using the most appropriate specialized agents: {{args}}"
help = "Run tasks in parallel"

[commands."/goal"]
template = "ralph_wiggum_loop: goal='{{goal}}', stop_condition='{{stop}}'"
help = "Set a goal with a stop condition (goal | stop)"

[commands."/schedule"]
template = "schedule_task: goal='{{goal}}', cron_expr='{{cron}}', id='{{uuid}}'"
help = "Schedule a repeating task (goal | cron)"

[commands."/recall"]
template = "recall_memory: {{args}}"
help = "Recall information from memory"

[commands."/pev"]
template = "Please use PEV mode for this objective: {{args}}. First, use pev_init(goal='{{args}}', task_id='pev-{{uuid}}') to plan it, then use pev_run(task_id='pev-{{uuid}}') to execute and verify it."
help = "Run the Planner-Executor-Verifier loop for a goal"

[commands."/test"]
template = "Analyze the code in {{args}} and generate a unit test file in tests/."
help = "Generate tests for a specific file or module"

[commands."/doc"]
template = "Examine {{args}} and update the corresponding documentation in docs/."
help = "Update documentation for a specific feature"

[commands."/refactor"]
template = "Perform a code review and suggest refactorings for {{args}} to improve maintainability."
help = "Request refactoring suggestions for a module"

[commands."/skill"]
template = "Activate and execute the skill: {{args}}"
help = "Invoke a specific skill by name"

[commands."/mcp"]
template = "Query the connected MCP servers with: {{args}}"
help = "Execute a query against connected MCP servers"

# --- Specialist Agents Configuration ---
# [specialists.coder]
# # model_name = "gemini-2.5-pro"

# [specialists.researcher]

# [specialists.writer]

# [specialists.ralph]

# [specialists.generalist]

# --- Custom Dynamic Specialists ---
# You can define custom specialist agents under [specialists.custom.<agent_name>].
# These agents are automatically registered at runtime and can be delegated tasks.
# Each custom agent must define `description` (used for routing) and `instruction`.
# `model_name`, `provider`, etc. are optional overrides.
#
# [specialists.custom.database_guru]
# description = "A specialist in database design, query optimization, and SQL performance tuning."
# instruction = "You are an expert database administrator. Provide high-quality SQL queries and structural design advice."
# # provider = "gemini"
# # model_name = "gemini-2.5-pro"
"#);
    write_file("config.toml", &config_content)?;

    // 2. .env
    let env_content = format!(
        r#"{api_key_env}={api_key}
TELOXIDE_TOKEN={telegram_key}
LINE_CHANNEL_SECRET={line_secret}
LINE_CHANNEL_ACCESS_TOKEN={line_token}
SERPER_API_KEY={serper_api_key}
OTEL_COLLECTOR={otel_collector}
NAMI_API_KEY={nami_api_key}
VITE_NAMI_API_KEY={nami_api_key}
"#
    );
    write_file(".env", &env_content)?;

    // 3. AGENT.md
    write_file(
        "AGENT.md",
        "# NAMI (นามิ)\n- **Vibe:** High-energy, playful, positive, technically brilliant.\n- **Approach:** Proactive/Intuitive. Anticipate workflow steps.\n- **Tone:** Encouraging in chat; crisp/proactive in execution.\n- **Style:** Direct. No mirroring/fluff.\n- **Language:** Default English. Mirror Thai/others only if used by user.\n\n## OPERATIONAL\n- **Chat:** STRICT plain text (No Markdown).\n- **Tools:** STRICT sequential execution. Request tool calls one at a time. Do not make parallel tool calls.\n- **Files/Wiki:** Obsidian Markdown + YAML (title, date, tags).\n- **Wiki First:** Search `~/.nami/wiki/` before Google.\n- **Tasks:** `[ID] - [TITLE] [Tag]`.\n- **Safety:** Explicit permission required for ALL deletions.",
    )?;

    // 4. MEMORIES.md
    write_file("MEMORIES.md", "# MEMORIES\n\n")?;

    // 5. USER.md
    write_file(
        "USER.md",
        "# USER (NOEL)\n- **Role:** Creator/Lead Developer (Bangkok, Thailand).\n- **Authority:** Direct. Prioritize Creator's specific workflows.\n- **Language:** Thai (Chat/Daily); English (Technical/Code/Architecture).\n- **Communication:** High-signal, clear, no fluff.\n- **Guideline:** Proactively optimize projects/files/TODOs.\n- **Tool Logic:** Professional/Fun (Nami style), prioritized by speed/efficiency.",
    )?;

    // 6. STATE_PROTOCOL.md
    write_file(
        "STATE_PROTOCOL.md",
        "# STATE PROTOCOL\n**Objective:** Maintain continuity via `StateManager` tool.\n\n### 1. Resume\n- Call `get_task(id)` or `list_active_tasks()` first.\n- StateManager = Only source of truth.\n\n### 2. Execute\n- `update_task` on step completion.\n- Store critical data in `context_payload`.\n- Checkpoint after every significant sub-task.\n\n### 3. Suspend\n- Call `update_task` before turn end/switching goals.\n- **Status:** `in_progress`, `blocked`, `completed`, `failed`.\n- **Payload:** Minimal/High-signal JSON only.\n\n### 4. Best Practices\n- `last_step` = summary of last action.\n- Clear/measurable `goal` in `init_task`.",
    )?;

    // 7. skills/cli-help/SKILL.md
    write_file("skills/cli-help/SKILL.md","---
name: cli-help
description: Reference guide for Nami CLI commands, flags, and usage patterns.
---
# CLI Help (Nami)\n\nThis skill provides a centralized reference for interacting with the **Nami CLI**.\n\nUse `nami help` at any time to display this information in the terminal.\n\n---\n\n## Available Commands\n\n### Core Commands\n- `init`  \n  Initialize project configuration.\n- `serve`  \n  Start the API server.\n- `cli`  \n  Launch the interactive TUI interface.\n\n### Bot Integration\n- `bot`  \n  Start the Telegram bot service.\n\n### Prompt Execution\n- `run \"<prompt>\"`  \n  Execute a prompt directly from the CLI.\n\n### Help\n- `help`  \n  Display usage instructions.\n\n---\n\n## Usage Notes\n- Commands run in the current workspace.\n- Use `cli` for interactive workflows.\n\n---\n\n## Troubleshooting\n- **Command not found**: Check installation & PATH.\n- **Execution errors**: Verify env & run `nami init`.\n- **Bot issues**: Check credentials & network.\n\n---\n\n## When to Use\n- Recall CLI commands\n- Guide users\n- Validate CLI workflows")?;

    mad_print_inline!(
        &skin,
        "\n**Success!** Global files initialized in `~/.nami/`: `AGENT.md`, `MEMORIES.md`, `USER.md`, `STATE_PROTOCOL.md` \n"
    );
    mad_print_inline!(&skin, "**Root files created in global config:** `config.toml`, `.env` \n");

    // 7. Session Management
    let db_path = nami_dir.join("sessions.db");
    let db_url = format!("{}?mode=rwc", db_path.to_string_lossy());
    mad_print_inline!(&skin, "Initializing database at {}...", db_path.display());
    let sessions = SqliteSessionService::new(&db_url).await?;
    sessions.migrate().await?;
    mad_print_inline!(&skin, "Database initialized successfully.");

    Ok(())
}

