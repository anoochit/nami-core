use crate::utils::get_nami_dir;

pub async fn run_switch_flow() -> anyhow::Result<Option<(String, String)>> {
    use inquire::Select;
    use inquire::Text;

    println!("\n🔄 Let's switch your LLM provider and model dynamically!");

    let providers = vec!["gemini", "openai", "anthropic", "ollama", "openrouter"];
    let selected_provider = Select::new("Choose LLM Provider:", providers).prompt()?;

    let standard_models = crate::utils::fetch_models_for_provider(selected_provider).await;

    let model_choice = Select::new(&format!("Choose model for {}:", selected_provider), standard_models).prompt()?;

    let final_model = if model_choice == "custom" {
        Text::new("Enter custom model name:").prompt()?
    } else {
        model_choice.to_string()
    };

    let default_env = crate::utils::provider_env_var(&selected_provider);

    let final_env = if !default_env.is_empty() {
        let env_prompt = Text::new("Enter Environment Variable Name for API Key:")
            .with_default(default_env)
            .prompt()?;
        Some(env_prompt)
    } else {
        None
    };

    if let Some(ref env_name) = final_env {
        use inquire::Password;
        let prompt_text = format!("Enter API Key value for {}:", env_name);
        let key_input = Password::new(&prompt_text)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        if !key_input.is_empty() {
            let env_path = get_nami_dir().join(".env");
            let mut lines = Vec::new();
            let mut updated = false;
            if env_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&env_path) {
                    for line in content.lines() {
                        if let Some(idx) = line.find('=') {
                            let k = line[..idx].trim();
                            if k == env_name {
                                lines.push(format!("{}={}", env_name, key_input));
                                updated = true;
                            } else {
                                lines.push(line.to_string());
                            }
                        } else {
                            lines.push(line.to_string());
                        }
                    }
                }
            }
            if !updated {
                lines.push(format!("{}={}", env_name, key_input));
            }
            if let Err(e) = std::fs::write(&env_path, lines.join("\n") + "\n") {
                println!("⚠️ Failed to write API key to .env: {}", e);
            } else {
                println!("✅ Successfully updated API key in ~/.nami/.env");
            }
        }
    }

    let config_path = get_nami_dir().join("config.toml");
    if config_path.exists() {
        if let Ok(config_str) = std::fs::read_to_string(&config_path) {
            if let Ok(mut toml_val) = toml::from_str::<toml::Value>(&config_str) {
                if let Some(model_table) = toml_val.get_mut("model") {
                    if let Some(table) = model_table.as_table_mut() {
                        table.insert("provider".to_string(), toml::Value::String(selected_provider.to_string()));
                        table.insert("model_name".to_string(), toml::Value::String(final_model.clone()));
                        if let Some(env) = final_env {
                            table.insert("api_key_env".to_string(), toml::Value::String(env));
                        } else {
                            table.remove("api_key_env");
                        }
                    }
                } else if let Some(root_table) = toml_val.as_table_mut() {
                    let mut model_table = toml::value::Table::new();
                    model_table.insert("provider".to_string(), toml::Value::String(selected_provider.to_string()));
                    model_table.insert("model_name".to_string(), toml::Value::String(final_model.clone()));
                    if let Some(env) = final_env {
                        model_table.insert("api_key_env".to_string(), toml::Value::String(env));
                    }
                    root_table.insert("model".to_string(), toml::Value::Table(model_table));
                }

                if let Ok(updated_str) = toml::to_string_pretty(&toml_val) {
                    if let Err(e) = std::fs::write(&config_path, updated_str) {
                        println!("⚠️ Failed to persist changes to config.toml: {}", e);
                    }
                }
            }
        }
    }

    Ok(Some((selected_provider.to_string(), final_model)))
}