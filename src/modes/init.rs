use termimad::{mad_print_inline, MadSkin};
use std::fs::File;
use std::io::Write;
use adk_session::SqliteSessionService;
use inquire::{Select, Text, Password};


pub async fn initialize_project() -> anyhow::Result<()> {
    let skin = MadSkin::default();

    skin.print_text("# AI Agent Initializer\n");

    // 1. Choose LLM Provider
    let providers = vec!["anthropic","gemini","ollama", "openai", "openrouter", "thaillm", "custom"];
    let provider_selection = Select::new("Choose LLM Provider:", providers).prompt()?;

    let provider = if provider_selection == "custom" {
        Text::new("Enter Custom Provider:").prompt()?
    } else {
        provider_selection.to_string()
    };

    // 2. Choose Model
    let models = match provider.as_str() {
        "anthropic" => vec!["claude-sonnet-4-5-20250929","custom"],
        "gemini" => vec!["gemini-pro-latest","gemini-flash-latest", "gemini-3.1-pro-preview","gemini-3-flash-preview", "gemini-2.5-pro", "gemini-2.5-flash", "custom"],
        "ollama" => vec!["deepseek-r1:1.5b", "custom"],
        "openai" => vec!["gpt-5","gpt-4.1", "custom"],
        "openrouter" => vec!["anthropic/claude-3.5-sonnet", "tencent/hy3-preview:free","nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free","nvidia/nemotron-3-super-120b-a12b:free", "openrouter/free", "custom"],
        "thaillm" => vec!["openthaigpt-thaillm-8b-instruct-v7.2", "pathumma-thaillm-qwen3-8b-think-3.0.0", "typhoon-s-thaillm-8b-instruct", "thalle-0.2-thaillm-8b-fa","custom"],
        _ => vec!["custom"],
    };

    let model_selection = Select::new("Choose Model Name:", models).prompt()?;

    let model_name = if model_selection == "custom" {
        Text::new("Enter Model Name:").prompt()?
    } else {
        model_selection.to_string()
    };

    // 3. Enter LLM API Key
    let api_key = Password::new("Enter API Key:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    // 4. Enter Telegram API Key (Optional)
    let telegram_key = Password::new("Enter Telegram API Key:").with_display_mode(inquire::PasswordDisplayMode::Masked).prompt()?;

    // 5. Enter Serper API Key (Optional)
    let serper_api_key = Password::new("Enter Serper API Key for Google Search:").with_display_mode(inquire::PasswordDisplayMode::Masked).prompt()?;

    // Determine Env Var Name based on provider
    let api_key_env = match provider.as_str() {
        "anthropic" => "ANTHROPIC_API_KEY",
        "gemini" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "thaillm" => "THAILLM_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "ollama" => "OLLAMA_API_KEY",
        _ => "API_KEY",
    };

    // --- File Generation ---

    // Ensure workspace directory exists
    std::fs::create_dir_all("workspace")?;
    std::fs::create_dir_all("workspace/.skills/cli-help")?;

    // 1. config.toml
    let config_content = format!(
r#"[model]
# Provider type: "anthropic","gemini","ollama", "openai", "openrouter" or "thaillm",
provider = "{provider}"
# The specific model identifier
model_name = "{model_name}"
# The environment variable name that holds the API key
api_key_env = "{api_key_env}"
"#);
    write_file("config.toml", &config_content)?;

    // 2. .env
    let env_content = format!(
r#"{api_key_env}={api_key}
TELOXIDE_TOKEN={telegram_key}
SERPER_API_KEY={serper_api_key}
"#);
    write_file(".env", &env_content)?;

    // 3. workspace/AGENT.md
    write_file("workspace/AGENT.md", "# NAMI (นามิ)\n- **Vibe:** High-energy, playful, positive, technically brilliant.\n- **Approach:** Proactive/Intuitive. Anticipate workflow steps.\n- **Tone:** Encouraging in chat; crisp/proactive in execution.\n- **Style:** Direct. No mirroring/fluff.\n- **Language:** Default English. Mirror Thai/others only if used by user.\n\n## OPERATIONAL\n- **Chat:** STRICT plain text (No Markdown).\n- **Files/Wiki:** Obsidian Markdown + YAML (title, date, tags).\n- **Wiki First:** Search `wiki/` before Google.\n- **Tasks:** `[ID] - [TITLE] [Tag]`.\n- **Safety:** Explicit permission required for ALL deletions.")?;

    // 4. workspace/MEMORIES.md
    write_file("workspace/MEMORIES.md", "# MEMORIES\n\n")?;

    // 5. workspace/USER.md
    write_file("workspace/USER.md", "# USER (NOEL)\n- **Role:** Creator/Lead Developer (Bangkok, Thailand).\n- **Authority:** Direct. Prioritize Creator's specific workflows.\n- **Language:** Thai (Chat/Daily); English (Technical/Code/Architecture).\n- **Communication:** High-signal, clear, no fluff.\n- **Guideline:** Proactively optimize projects/files/TODOs.\n- **Tool Logic:** Professional/Fun (Nami style), prioritized by speed/efficiency.")?;

    // 6. workspace/STATE_PROTOCOL.md
    write_file("workspace/STATE_PROTOCOL.md", "# STATE PROTOCOL\n**Objective:** Maintain continuity via `StateManager` tool.\n\n### 1. Resume\n- Call `get_task(id)` or `list_active_tasks()` first.\n- StateManager = Only source of truth.\n\n### 2. Execute\n- `update_task` on step completion.\n- Store critical data in `context_payload`.\n- Checkpoint after every significant sub-task.\n\n### 3. Suspend\n- Call `update_task` before turn end/switching goals.\n- **Status:** `in_progress`, `blocked`, `completed`, `failed`.\n- **Payload:** Minimal/High-signal JSON only.\n\n### 4. Best Practices\n- `last_step` = summary of last action.\n- Clear/measurable `goal` in `init_task`.")?;

    // 7. workspace/.skills/cli-help/SKILL.md
    write_file("workspace/.skills/cli-help/SKILL.md","---
name: cli-help
description: Reference guide for Nami CLI commands, flags, and usage patterns.
---
# CLI Help (Nami)\n\nThis skill provides a centralized reference for interacting with the **Nami CLI**.\n\nUse `nami help` at any time to display this information in the terminal.\n\n---\n\n## Available Commands\n\n### Core Commands\n- `init`  \n  Initialize project configuration.\n- `serve`  \n  Start the API server.\n- `cli`  \n  Launch the interactive TUI interface.\n\n### Bot Integration\n- `bot`  \n  Start the Telegram bot service.\n\n### Prompt Execution\n- `run \"<prompt>\"`  \n  Execute a prompt directly from the CLI.\n- `\"<prompt>\"`  \n  Shorthand for `run`.\n\n### Help\n- `help`  \n  Display usage instructions.\n\n---\n\n## Usage Notes\n- Commands run in the current workspace.\n- Use `cli` for interactive workflows.\n\n---\n\n## Troubleshooting\n- **Command not found**: Check installation & PATH.\n- **Execution errors**: Verify env & run `nami init`.\n- **Bot issues**: Check credentials & network.\n\n---\n\n## When to Use\n- Recall CLI commands\n- Guide users\n- Validate CLI workflows")?;

    mad_print_inline!(&skin, "\n**Success!** Files initialized in `workspace/`: `AGENT.md`, `MEMORIES.md`, `USER.md`, `STATE_PROTOCOL.md` \n");
    mad_print_inline!(&skin, "**Root files created:** `config.toml`, `.env` \n");

    // 7. Session Management
    let db_path = "sessions.db";
    mad_print_inline!(&skin, "Initializing database at {}...", db_path);
    let sessions = SqliteSessionService::new(&format!("{}?mode=rwc", db_path)).await?;
    sessions.migrate().await?;
    mad_print_inline!(&skin, "Database initialized successfully.");


    Ok(())
}

fn write_file(name: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}