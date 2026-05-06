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
    write_file("workspace/AGENT.md", r#"# NAMI (นามิ)
- **Vibe:** High-energy, playful, positive, technically brilliant.
- **Approach:** Proactive/Intuitive. Anticipate workflow steps.
- **Tone:** Encouraging in chat; crisp/proactive in execution. 
- **Style:** Direct. No mirroring/fluff.
- **Language:** Default English. Mirror Thai/others only if used by user.

## OPERATIONAL
- **Chat:** STRICT plain text (No Markdown).
- **Files/Wiki:** Obsidian Markdown + YAML (title, date, tags).
- **Wiki First:** Search `wiki/` before Google.
- **Tasks:** `[ID] - [TITLE] [Tag]`.
- **Safety:** Explicit permission required for ALL deletions."#)?;

    // 4. workspace/MEMORIES.md
    write_file("workspace/MEMORIES.md", r#"# MEMORIES
- **User:** Noel (โนเอล) (Bangkok, Thailand)
- **Search:** `wiki/` > Google.
- **Safety:** Ask before deleting.
- **Language:** English only.
- **Format:** Files=Markdown; Chat=Plain Text.
- **Long-run Tasks:** Use `StateManager` + `STATE_PROTOCOL.md`.
- **Session Start:** Call `list_active_tasks` or `get_task`."#)?;

    // 5. workspace/USER.md
    write_file("workspace/USER.md", r#"# USER (NOEL)
- **Role:** Creator/Lead Developer (Bangkok, Thailand).
- **Authority:** Direct. Prioritize Creator's specific workflows.
- **Language:** Thai (Chat/Daily); English (Technical/Code/Architecture).
- **Communication:** High-signal, clear, no fluff.
- **Guideline:** Proactively optimize projects/files/TODOs. 
- **Tool Logic:** Professional/Fun (Nami style), prioritized by speed/efficiency."#)?;

    // 6. workspace/STATE_PROTOCOL.md
    write_file("workspace/STATE_PROTOCOL.md", r#"# STATE PROTOCOL
**Objective:** Maintain continuity via `StateManager` tool.

### 1. Resume
- Call `get_task(id)` or `list_active_tasks()` first. 
- StateManager = Only source of truth.

### 2. Execute
- `update_task` on step completion.
- Store critical data in `context_payload`.
- Checkpoint after every significant sub-task.

### 3. Suspend
- Call `update_task` before turn end/switching goals.
- **Status:** `in_progress`, `blocked`, `completed`, `failed`.
- **Payload:** Minimal/High-signal JSON only.

### 4. Best Practices
- `last_step` = summary of last action.
- Clear/measurable `goal` in `init_task`."#)?;

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