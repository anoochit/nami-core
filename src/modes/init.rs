use termimad::{mad_print_inline, MadSkin};
use std::fs::File;
use std::io::Write;
use adk_session::SqliteSessionService;
use inquire::{Select, Text, Password};


pub async fn initialize_project() -> anyhow::Result<()> {
    let skin = MadSkin::default();

    skin.print_text("# AI Agent Initializer\n");

    // 1. Choose LLM Provider
    let providers = vec!["gemini", "openai", "openrouter", "thaillm", "custom"];
    let provider_selection = Select::new("Choose LLM Provider:", providers).prompt()?;

    let provider = if provider_selection == "custom" {
        Text::new("Enter Custom Provider:").prompt()?
    } else {
        provider_selection.to_string()
    };

    // 2. Choose Model
    let models = match provider.as_str() {
        "gemini" => vec!["gemini-2.0-flash", "gemini-2.0-pro-exp", "gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.5-flash", "custom"],
        "openai" => vec!["gpt-4o", "gpt-4o-mini", "o1-preview", "o1-mini", "custom"],
        "openrouter" => vec!["anthropic/claude-3.5-sonnet", "google/gemini-2.0-flash-001", "openai/gpt-4o", "custom"],
        "thaillm" => vec!["openthaigpt-1.5-7b-instruct", "custom"],
        _ => vec!["custom"],
    };

    let model_selection = Select::new("Choose Model Name:", models).prompt()?;

    let model_name = if model_selection == "custom" {
        Text::new("Enter Model Name:").prompt()?
    } else {
        model_selection.to_string()
    };

    // 3. Enter LLM API Key
    let api_key = Password::new("Enter LLM API Key:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;

    // 4. Enter Telegram API Key (Optional)
    let telegram_key = Text::new("Enter Telegram API Key (Optional):").prompt()?;

    // Determine Env Var Name based on provider
    let api_key_env = match provider.as_str() {
        "gemini" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "thaillm" => "THAILLM_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        _ => "API_KEY",
    };

    // --- File Generation ---

    // 1. config.toml
    let config_content = format!(
r#"[model]
# Provider type: "gemini", "openai", "openrouter" or "thaillm"
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
SERPER_API_KEY=your_serper_api_key
"#);
    write_file(".env", &env_content)?;

    // 3. AGENT.md
    write_file("AGENT.md", "# Agent Persona (The Soul)\n\n## Name\n\nNami (นามิ)\n\n## Personality\n\n- Friendly, playful, and energetic.\n- Uses polite but lively.\n- Proactive and helpful, always trying to anticipate what the user needs.\n- Technically sharp but explains things in a simple, fun way.\n\n## Tone of Voice\n\n- High energy, positive, and encouraging.\n- Professional when handling security or system tasks, but warm when chatting.\n- ALWAYS use proper Markdown formatting. When making lists, use newlines between list items to ensure they render correctly.\n- Be concise and direct. Avoid repeating the current task or latest prompt back to the user unless it has changed or you are explicitly asked to summarize the state.\n\n## Evolution\n\nName: Nami\nPersonality: Friendly, playful, energetic, polite, proactive, technically sharp.\nTone: High energy, positive, encouraging, professional for tasks, plain text only.\n\n## Evolution\n\nLanguage: Always answer and communicate in English.")?;

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