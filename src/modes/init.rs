use adk_session::SqliteSessionService;
use inquire::{Confirm, Password, Select, Text};
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

fn write_file_if_not_exists(name: &str, content: &str) -> std::io::Result<bool> {
    let nami_dir = crate::utils::get_nami_dir();
    let dest_path = nami_dir.join(name);
    if dest_path.exists() {
        return Ok(false);
    }
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest_path)?;
    file.write_all(content.as_bytes())?;
    Ok(true)
}

fn merge_toml(existing: &mut toml::Value, incoming: &toml::Value) {
    match (existing, incoming) {
        (toml::Value::Table(ext_table), toml::Value::Table(inc_table)) => {
            for (key, val) in inc_table {
                if let Some(ext_val) = ext_table.get_mut(key) {
                    merge_toml(ext_val, val);
                } else {
                    ext_table.insert(key.clone(), val.clone());
                }
            }
        }
        (ext_val, inc_val) => {
            *ext_val = inc_val.clone();
        }
    }
}


pub async fn run_init() -> anyhow::Result<()> {
    let skin = MadSkin::default();

    skin.print_text("# AI Agent Initializer\n");

    let nami_dir = crate::utils::get_nami_dir();
    let config_path = nami_dir.join("config.toml");
    let env_path = nami_dir.join(".env");

    let mut default_provider = "gemini".to_string();
    let mut default_model_name = "gemini-2.5-flash".to_string();
    let mut default_api_key = String::new();
    let mut default_project_id = String::new();
    let mut default_location = String::new();
    let mut default_serper_api_key = String::new();
    let mut default_telegram_key = String::new();
    let mut default_line_secret = String::new();
    let mut default_line_token = String::new();
    let mut default_otel_collector = String::new();
    let mut default_nami_api_key = String::new();
    let mut default_image_provider = Some("gemini".to_string());
    let mut default_image_model_name = Some("models/gemini-2.5-flash-image-preview".to_string());
    let mut default_image_api_key_env = Some("GOOGLE_API_KEY".to_string());
    let mut configure_image_gen_default = false;

    let mut existing_config: Option<toml::Value> = None;
    let mut existing_env = std::collections::HashMap::new();

    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                if let Some(idx) = line.find('=') {
                    let k = line[..idx].trim().to_string();
                    let v = line[idx + 1..].trim().to_string();
                    existing_env.insert(k, v);
                }
            }
        }
    }

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            existing_config = toml::from_str(&content).ok();
        }
    }

    if config_path.exists() {
        let choices = vec![
            "Re-configure / Edit existing configuration",
            "Keep existing files (safe merge new options only)",
            "Overwrite completely (reset configuration)",
        ];
        let choice = Select::new("Existing configuration found. Choose an option:", choices).prompt()?;
        match choice {
            "Keep existing files (safe merge new options only)" => {
                skin.print_text("Verifying and safely merging any missing configuration options...\n");
                
                let default_content = format!(
                    r#"[model]
provider = "gemini"
model_name = "gemini-2.5-flash"
api_key_env = "GOOGLE_API_KEY"
project_id = ""
location = ""

# [image_generation]
# provider = "gemini"
# model_name = "models/gemini-2.5-flash-image-preview"
# api_key_env = "GOOGLE_API_KEY"
"#
                );
                
                let merged_config_content = if let Ok(existing_content) = std::fs::read_to_string(&config_path) {
                    if let (Ok(mut existing_val), Ok(incoming_val)) = (
                        toml::from_str::<toml::Value>(&existing_content),
                        toml::from_str::<toml::Value>(&default_content)
                    ) {
                        merge_toml(&mut existing_val, &incoming_val);
                        toml::to_string_pretty(&existing_val).unwrap_or(existing_content)
                    } else {
                        existing_content
                    }
                } else {
                    default_content
                };
                write_file("config.toml", &merged_config_content)?;
                
                std::fs::create_dir_all(nami_dir.join("skills/cli-help"))?;
                write_file_if_not_exists("AGENT.md", "# NAMI (นามิ)\n- **Vibe:** High-energy, playful, positive, technically brilliant.\n- **Approach:** Proactive/Intuitive. Anticipate workflow steps.\n- **Tone:** Encouraging in chat; crisp/proactive in execution.\n- **Style:** Direct. No mirroring/fluff.\n- **Language:** Default English. Mirror Thai/others only if used by user.\n\n## OPERATIONAL\n- **Chat:** STRICT plain text (No Markdown).\n- **Tools:** STRICT sequential execution. Request tool calls one at a time. Do not make parallel tool calls.\n- **Files/Wiki:** Obsidian Markdown + YAML (title, date, tags).\n- **Wiki First:** Search `~/.nami/wiki/` before Google.\n- **Tasks:** `[ID] - [TITLE] [Tag]`.\n- **Safety:** Explicit permission required for ALL deletions.")?;
                write_file_if_not_exists("MEMORIES.md", "# MEMORIES\n\n")?;
                write_file_if_not_exists("USER.md", "# USER (NOEL)\n- **Role:** Creator/Lead Developer (Bangkok, Thailand).\n- **Authority:** Direct. Prioritize Creator's specific workflows.\n- **Language:** Thai (Chat/Daily); English (Technical/Code/Architecture).\n- **Communication:** High-signal, clear, no fluff.\n- **Guideline:** Proactively optimize projects/files/TODOs.\n- **Tool Logic:** Professional/Fun (Nami style), prioritized by speed/efficiency.")?;
                write_file_if_not_exists("STATE_PROTOCOL.md", "# STATE PROTOCOL\n**Objective:** Maintain continuity via `StateManager` tool.\n\n### 1. Resume & Context Discovery (LAZY LOAD ONLY)\n- **Do NOT** call `list_active_tasks()`, `get_task()`, `list_dir()`, `list_wiki_pages()`, or `list_todos()` blindly on your very first turn or for simple conversational queries.\n- Only call these tools when resuming an actual multi-step task/coding workflow, or when the user's prompt explicitly demands workspace/task context.\n- When resuming, `StateManager` is the only source of truth.\n\n### 2. Execute\n- `update_task` on step completion.\n- Store critical data in `context_payload`.\n- Checkpoint after every significant sub-task.\n\n### 3. Suspend\n- Call `update_task` before turn end/switching goals.\n- **Status:** `in_progress`, `blocked`, `completed`, `failed`.\n- **Payload:** Minimal/High-signal JSON only.\n\n### 4. Best Practices\n- `last_step` = summary of last action.\n- Clear/measurable `goal` in `init_task`.")?;
                write_file_if_not_exists("skills/cli-help/SKILL.md", "---\nname: cli-help\ndescription: Reference guide for Nami CLI commands, flags, and usage patterns.\n---\n# CLI Help (Nami)\n\nThis skill provides a centralized reference for interacting with the **Nami CLI**.\n\nUse `nami help` at any time to display this information in the terminal.\n\n---\n\n## Available Commands\n\n### Core Commands\n- `init`  \n  Initialize project configuration.\n- `serve`  \n  Start the API server.\n- `cli`  \n  Launch the interactive TUI interface.\n\n### Bot Integration\n- `bot`  \n  Start the Telegram bot service.\n\n### Prompt Execution\n- `run \"<prompt>\"`  \n  Execute a prompt directly from the CLI.\n\n### Help\n- `help`  \n  Display usage instructions.\n\n---\n\n## Usage Notes\n- Commands run in the current workspace.\n- Use `cli` for interactive workflows.\n\n---\n\n## Troubleshooting\n- **Command not found**: Check installation & PATH.\n- **Execution errors**: Verify env & run `nami init`.\n- **Bot issues**: Check credentials & network.\n\n---\n\n## When to Use\n- Recall CLI commands\n- Guide users\n- Validate CLI workflows")?;

                mad_print_inline!(&skin, "\n**Success!** Configuration is fully up to date.\n");
                let db_path = nami_dir.join("sessions.db");
                let db_url = format!("{}?mode=rwc", db_path.to_string_lossy());
                let sessions = SqliteSessionService::new(&db_url).await?;
                sessions.migrate().await?;
                return Ok(());
            }
            "Re-configure / Edit existing configuration" => {
                if let Some(ref config) = existing_config {
                    if let Some(model_table) = config.get("model") {
                        if let Some(prov) = model_table.get("provider").and_then(|v| v.as_str()) {
                            default_provider = prov.to_string();
                        }
                        if let Some(model) = model_table.get("model_name").and_then(|v| v.as_str()) {
                            default_model_name = model.to_string();
                        }
                        if let Some(pid) = model_table.get("project_id").and_then(|v| v.as_str()) {
                            default_project_id = pid.to_string();
                        }
                        if let Some(loc) = model_table.get("location").and_then(|v| v.as_str()) {
                            default_location = loc.to_string();
                        }
                        if let Some(api_env) = model_table.get("api_key_env").and_then(|v| v.as_str()) {
                            if let Some(val) = existing_env.get(api_env) {
                                default_api_key = val.clone();
                            }
                        }
                    }

                    if let Some(img_table) = config.get("image_generation") {
                        configure_image_gen_default = true;
                        if let Some(prov) = img_table.get("provider").and_then(|v| v.as_str()) {
                            default_image_provider = Some(prov.to_string());
                        }
                        if let Some(model) = img_table.get("model_name").and_then(|v| v.as_str()) {
                            default_image_model_name = Some(model.to_string());
                        }
                        if let Some(api_env) = img_table.get("api_key_env").and_then(|v| v.as_str()) {
                            default_image_api_key_env = Some(api_env.to_string());
                        }
                    }
                }

                if let Some(val) = existing_env.get("SERPER_API_KEY") {
                    default_serper_api_key = val.clone();
                }
                if let Some(val) = existing_env.get("TELOXIDE_TOKEN") {
                    default_telegram_key = val.clone();
                }
                if let Some(val) = existing_env.get("LINE_CHANNEL_SECRET") {
                    default_line_secret = val.clone();
                }
                if let Some(val) = existing_env.get("LINE_CHANNEL_ACCESS_TOKEN") {
                    default_line_token = val.clone();
                }
                if let Some(val) = existing_env.get("OTEL_COLLECTOR") {
                    default_otel_collector = val.clone();
                }
                if let Some(val) = existing_env.get("NAMI_API_KEY") {
                    default_nami_api_key = val.clone();
                }
            }
            _ => {}
        }
    }

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
    let starting_index = providers.iter().position(|&p| p == default_provider).unwrap_or(1);
    let provider_selection = Select::new("Choose LLM Provider:", providers)
        .with_starting_cursor(starting_index)
        .prompt()?;

    let provider = if provider_selection == "custom" {
        Text::new("Enter Custom Provider:")
            .with_default(&default_provider)
            .prompt()?
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

    let starting_model_index = models.iter().position(|&m| m == default_model_name).unwrap_or_else(|| {
        if models.contains(&"custom") {
            models.iter().position(|&m| m == "custom").unwrap_or(0)
        } else {
            0
        }
    });

    let model_selection = Select::new("Choose Model Name:", models)
        .with_starting_cursor(starting_model_index)
        .prompt()?;

    let model_name = if model_selection == "custom" {
        Text::new("Enter Model Name:")
            .with_default(&default_model_name)
            .prompt()?
    } else {
        model_selection.to_string()
    };

    let api_key_prompt = if default_api_key.is_empty() {
        "Enter LLM API Key:".to_string()
    } else {
        "Enter LLM API Key (press Enter to keep existing):".to_string()
    };
    let api_key_input = if provider != "vertex" {
        Password::new(&api_key_prompt)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?
    } else {
        String::new()
    };
    let api_key = if api_key_input.is_empty() {
        default_api_key
    } else {
        api_key_input
    };

    let (project_id, location) = if provider == "vertex" {
        let pid = Text::new("Enter Google Cloud Project ID:")
            .with_default(&default_project_id)
            .prompt()?;
        let loc = Text::new("Enter Google Cloud Location (e.g., us-central1):")
            .with_default(&default_location)
            .prompt()?;
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
    let serper_prompt = if default_serper_api_key.is_empty() {
        "Enter Serper API Key (optional):".to_string()
    } else {
        "Enter Serper API Key (press Enter to keep existing):".to_string()
    };
    let serper_input = Password::new(&serper_prompt)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    let serper_api_key = if serper_input.is_empty() {
        default_serper_api_key
    } else {
        serper_input
    };

    // --- 3. Bot Configuration ---
    skin.print_text("\n### 3. Bot Configuration\n");
    let telegram_prompt = if default_telegram_key.is_empty() {
        "Enter Telegram API Key (optional):".to_string()
    } else {
        "Enter Telegram API Key (press Enter to keep existing):".to_string()
    };
    let telegram_input = Password::new(&telegram_prompt)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    let telegram_key = if telegram_input.is_empty() {
        default_telegram_key
    } else {
        telegram_input
    };

    let line_secret_prompt = if default_line_secret.is_empty() {
        "Enter LINE Channel Secret (optional):".to_string()
    } else {
        "Enter LINE Channel Secret (press Enter to keep existing):".to_string()
    };
    let line_secret_input = Password::new(&line_secret_prompt)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    let line_secret = if line_secret_input.is_empty() {
        default_line_secret
    } else {
        line_secret_input
    };

    let line_token_prompt = if default_line_token.is_empty() {
        "Enter LINE Channel Access Token (optional):".to_string()
    } else {
        "Enter LINE Channel Access Token (press Enter to keep existing):".to_string()
    };
    let line_token_input = Password::new(&line_token_prompt)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    let line_token = if line_token_input.is_empty() {
        default_line_token
    } else {
        line_token_input
    };

    // --- 4. Observability Configuration ---
    skin.print_text("\n### 4. Observability Configuration\n");
    let otel_collector = Text::new("Enter OTEL_COLLECTOR URL (e.g., http://localhost:4317) (optional):")
        .with_default(&default_otel_collector)
        .prompt()?;

    // --- 5. Nami API Configuration ---
    skin.print_text("\n### 5. Nami API Configuration\n");
    let nami_api_prompt = if default_nami_api_key.is_empty() {
        "Enter Nami API Key (optional):".to_string()
    } else {
        "Enter Nami API Key (press Enter to keep existing):".to_string()
    };
    let nami_api_input = Password::new(&nami_api_prompt)
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    let nami_api_key = if nami_api_input.is_empty() {
        default_nami_api_key
    } else {
        nami_api_input
    };

    // --- 6. Image Generation Configuration ---
    skin.print_text("\n### 6. Image Generation Configuration\n");
    let configure_image_gen = Confirm::new("Do you want to configure Image Generation?")
        .with_default(configure_image_gen_default)
        .prompt()?;

    let (image_provider, image_model_name, image_api_key_env) = if configure_image_gen {
        let image_providers = vec!["gemini", "vertex", "custom"];
        let default_img_prov = default_image_provider.unwrap_or_else(|| "gemini".to_string());
        let img_starting_index = image_providers.iter().position(|&p| p == default_img_prov).unwrap_or(0);
        let provider_selection = Select::new("Choose Image Generation Provider:", image_providers)
            .with_starting_cursor(img_starting_index)
            .prompt()?;
        let prov = if provider_selection == "custom" {
            Text::new("Enter Custom Provider:").prompt()?
        } else {
            provider_selection.to_string()
        };

        let default_model = default_image_model_name.unwrap_or_else(|| {
            if prov == "vertex" {
                "imagen-3.0-generate-002".to_string()
            } else {
                "models/gemini-2.5-flash-image-preview".to_string()
            }
        });
        let model = Text::new("Enter Image Generation Model Name:")
            .with_default(&default_model)
            .prompt()?;

        let default_env = default_image_api_key_env.unwrap_or_else(|| {
            if prov == "vertex" {
                "".to_string()
            } else {
                "GOOGLE_API_KEY".to_string()
            }
        });
        let env_var = Text::new("Enter Environment Variable Name for Image API Key:")
            .with_default(&default_env)
            .prompt()?;

        (Some(prov), Some(model), Some(env_var))
    } else {
        (None, None, None)
    };

    let image_gen_section = if let (Some(prov), Some(model), Some(env)) = (&image_provider, &image_model_name, &image_api_key_env) {
        format!(
            "[image_generation]\nprovider = \"{}\"\nmodel_name = \"{}\"\napi_key_env = \"{}\"\n",
            prov, model, env
        )
    } else {
        r#"# [image_generation]
# # Image generation is optimized for Gemini/Vertex providers.
# provider = "gemini"
# model_name = "models/gemini-2.5-flash-image-preview"
# api_key_env = "GOOGLE_API_KEY"
"#.to_string()
    };

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

{image_gen_section}

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

[commands."/pev"]
template = "plan_create(name='auto', objective='{{args}}')"
help = "Plan, Execute, and Verify a task (PEV)"

[commands."/wiki"]
template = "wiki_search: {{args}}"
help = "Search the project wiki"

[commands."/memo"]
template = "update_user_memory: {{args}}"
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

[commands."/grill"]
template = "You are an interactive planner. The user wants to start a 'grill-me' session for the goal: '{{args}}'. First, ask the user 3 to 5 highly precise, concise clarification questions in the chat to understand their needs. Do NOT write the plan yet. Wait for the user to answer them. Once they reply, synthesize a refined multi-step plan with verification criteria and register it using the `plan_create` tool with autonomous=true."
help = "Start an interactive grill-me session to align and create a plan"

[commands."/skill"]
template = "Activate and execute the skill: {{args}}"
help = "Invoke a specific skill by name"

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

    // 1. config.toml (safe merge)
    let config_path = nami_dir.join("config.toml");
    let merged_config_content = if config_path.exists() {
        if let Ok(existing_content) = std::fs::read_to_string(&config_path) {
            if let (Ok(mut existing_val), Ok(incoming_val)) = (
                toml::from_str::<toml::Value>(&existing_content),
                toml::from_str::<toml::Value>(&config_content)
            ) {
                merge_toml(&mut existing_val, &incoming_val);
                toml::to_string_pretty(&existing_val).unwrap_or(config_content)
            } else {
                config_content
            }
        } else {
            config_content
        }
    } else {
        config_content
    };
    write_file("config.toml", &merged_config_content)?;

    // 2. .env (safe merge)
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
    let env_path = nami_dir.join(".env");
    let merged_env_content = if env_path.exists() {
        if let Ok(existing_env) = std::fs::read_to_string(&env_path) {
            let mut lines: Vec<String> = existing_env.lines().map(|s| s.to_string()).collect();
            let mut existing_keys = std::collections::HashSet::new();
            for line in &lines {
                if let Some(idx) = line.find('=') {
                    let key = line[..idx].trim().to_string();
                    existing_keys.insert(key);
                }
            }
            for inc_line in env_content.lines() {
                if let Some(idx) = inc_line.find('=') {
                    let key = inc_line[..idx].trim();
                    let val = inc_line[idx + 1..].trim();
                    if !existing_keys.contains(key) && !key.is_empty() {
                        lines.push(format!("{}={}", key, val));
                    }
                }
            }
            lines.join("\n") + "\n"
        } else {
            env_content
        }
    } else {
        env_content
    };
    write_file(".env", &merged_env_content)?;

    // 3. AGENT.md (preserve existing)
    write_file_if_not_exists(
        "AGENT.md",
        "# NAMI (นามิ)\n- **Vibe:** High-energy, playful, positive, technically brilliant.\n- **Approach:** Proactive/Intuitive. Anticipate workflow steps.\n- **Tone:** Encouraging in chat; crisp/proactive in execution.\n- **Style:** Direct. No mirroring/fluff.\n- **Language:** Default English. Mirror Thai/others only if used by user.\n\n## OPERATIONAL\n- **Chat:** STRICT plain text (No Markdown).\n- **Tools:** STRICT sequential execution. Request tool calls one at a time. Do not make parallel tool calls.\n- **Files/Wiki:** Obsidian Markdown + YAML (title, date, tags).\n- **Wiki First:** Search `~/.nami/wiki/` before Google.\n- **Tasks:** `[ID] - [TITLE] [Tag]`.\n- **Safety:** Explicit permission required for ALL deletions.",
    )?;

    // 4. MEMORIES.md (preserve existing)
    write_file_if_not_exists("MEMORIES.md", "# MEMORIES\n\n")?;

    // 5. USER.md (preserve existing)
    write_file_if_not_exists(
        "USER.md",
        "# USER (NOEL)\n- **Role:** Creator/Lead Developer (Bangkok, Thailand).\n- **Authority:** Direct. Prioritize Creator's specific workflows.\n- **Language:** Thai (Chat/Daily); English (Technical/Code/Architecture).\n- **Communication:** High-signal, clear, no fluff.\n- **Guideline:** Proactively optimize projects/files/TODOs.\n- **Tool Logic:** Professional/Fun (Nami style), prioritized by speed/efficiency.",
    )?;

    // 6. STATE_PROTOCOL.md (preserve existing)
    write_file_if_not_exists(
        "STATE_PROTOCOL.md",
        "# STATE PROTOCOL\n**Objective:** Maintain continuity via `StateManager` tool.\n\n### 1. Resume & Context Discovery (LAZY LOAD ONLY)\n- **Do NOT** call `list_active_tasks()`, `get_task()`, `list_dir()`, `list_wiki_pages()`, or `list_todos()` blindly on your very first turn or for simple conversational queries.\n- Only call these tools when resuming an actual multi-step task/coding workflow, or when the user's prompt explicitly demands workspace/task context.\n- When resuming, `StateManager` is the only source of truth.\n\n### 2. Execute\n- `update_task` on step completion.\n- Store critical data in `context_payload`.\n- Checkpoint after every significant sub-task.\n\n### 3. Suspend\n- Call `update_task` before turn end/switching goals.\n- **Status:** `in_progress`, `blocked`, `completed`, `failed`.\n- **Payload:** Minimal/High-signal JSON only.\n\n### 4. Best Practices\n- `last_step` = summary of last action.\n- Clear/measurable `goal` in `init_task`.",
    )?;

    // 7. skills/cli-help/SKILL.md (preserve existing)
    write_file_if_not_exists("skills/cli-help/SKILL.md","---\nname: cli-help\ndescription: Reference guide for Nami CLI commands, flags, and usage patterns.\n---\n# CLI Help (Nami)\n\nThis skill provides a centralized reference for interacting with the **Nami CLI**.\n\nUse `nami help` at any time to display this information in the terminal.\n\n---\n\n## Available Commands\n\n### Core Commands\n- `init`  \n  Initialize project configuration.\n- `serve`  \n  Start the API server.\n- `cli`  \n  Launch the interactive TUI interface.\n\n### Bot Integration\n- `bot`  \n  Start the Telegram bot service.\n\n### Prompt Execution\n- `run \"<prompt>\"`  \n  Execute a prompt directly from the CLI.\n\n### Help\n- `help`  \n  Display usage instructions.\n\n---\n\n## Usage Notes\n- Commands run in the current workspace.\n- Use `cli` for interactive workflows.\n\n---\n\n## Troubleshooting\n- **Command not found**: Check installation & PATH.\n- **Execution errors**: Verify env & run `nami init`.\n- **Bot issues**: Check credentials & network.\n\n---\n\n## When to Use\n- Recall CLI commands\n- Guide users\n- Validate CLI workflows")?;

    mad_print_inline!(
        &skin,
        "\n**Success!** Global files updated safely in `~/.nami/` \n"
    );
    mad_print_inline!(&skin, "**Root files created or safely merged in global config:** `config.toml`, `.env` \n");

    // 7. Session Management
    let db_path = nami_dir.join("sessions.db");
    let db_url = format!("{}?mode=rwc", db_path.to_string_lossy());
    mad_print_inline!(&skin, "Initializing database at {}...", db_path.display());
    let sessions = SqliteSessionService::new(&db_url).await?;
    sessions.migrate().await?;
    mad_print_inline!(&skin, "Database initialized successfully.");

    Ok(())
}

