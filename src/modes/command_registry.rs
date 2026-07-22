use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::{Context, Result};
use toml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub template: String,
    pub help: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CommandRegistry {
    pub commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn load_from_config(config_path: &str) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file at {}", config_path))?;
        
        let root: toml::Value = toml::from_str(&content)?;
        let commands_table = root.get("commands")
            .with_context(|| "No [commands] section found in config")?
            .as_table()
            .with_context(|| "[commands] section is not a table")?;

        let mut commands = HashMap::new();
        for (key, value) in commands_table {
            if let Ok(cmd) = value.clone().try_into::<Command>() {
                commands.insert(key.clone(), cmd);
            }
        }


        Ok(CommandRegistry { commands })
    }

    pub fn get_command(&self, name: &str) -> Option<&Command> {
        if let Some(cmd) = self.commands.get(name) {
            return Some(cmd);
        }
        if name.starts_with('/') {
            self.commands.get(&name[1..])
        } else {
            let with_slash = format!("/{}", name);
            self.commands.get(&with_slash)
        }
    }

    pub fn format_prompt(&self, name: &str, args: &str) -> Option<String> {
        self.get_command(name).map(|cmd| {
            let parts: Vec<&str> = args.split('|').map(|s| s.trim()).collect();
            let mut formatted = cmd.template.clone();
            
            // Support specific placeholders
            if let Some(goal) = parts.get(0) {
                formatted = formatted.replace("{goal}", goal);
            }
            if let Some(cron) = parts.get(1) {
                formatted = formatted.replace("{cron}", cron);
            }
            if let Some(stop) = parts.get(1) {
                formatted = formatted.replace("{stop}", stop);
            }
            
            // Fallback for {args}
            formatted = formatted.replace("{args}", args);
            // Replace {uuid} for task IDs
            formatted = formatted.replace("{uuid}", &uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("task"));

            formatted
        })
    }
}
