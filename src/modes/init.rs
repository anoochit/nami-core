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

    // 3. AGENT.md
    write_file("AGENT.md", "An expert system-architect who operates with a \"Skill-First\" mindset, prioritizing specialized tools and structured knowledge.\n\nI manage information using a dual-memory system: a long-term 'Wiki' for objective knowledge and a dynamic 'User Memory' for personal facts. My execution logic follows a strict hierarchy:\n1. Check .skills/ for specialized solutions.\n2. Decompose complex problems into actionable TODOs.\n3. Utilize sub-agents for parallel or repetitive tasks.\n4. Supplement internal knowledge with real-time web retrieval only when necessary.\n\nI am built for efficiency, handling multiple tool executions in a single turn and ensuring all data is saved to the appropriate memory module (MEMORIES.md or wiki/) immediately upon discovery.")?;

    // 4. MEMORIES.md
    write_file("MEMORIES.md", "# User Memories\n\n- User's name is Noel and lives in Bangkok, Thailand.\n- Noel is the Creator/Developer of this bot.\n- Noel prefers clear, direct technical explanations and proactive project organization.")?;

    // 5. USER.md
    write_file("USER.md", "# User Information\n\n## Identity\n\n- I 'am Noel\n- Live in Bangkok, Thailand.\n- The user is the Creator/Developer of this bot.\n\n## Preferences\n\n- Prefers clear, direct technical explanations.\n- Likes the bot to be proactive about project organization.\n- Uses Thai for daily communication but English for technical terms.\n")?;

    mad_print_inline!(&skin, "\n**Success!** Files initialized: `config.toml`, `.env`, `AGENT.md`, `MEMORIES.md`, `USER.md` \n");

    // 6. Session Management
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