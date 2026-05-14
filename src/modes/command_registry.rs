use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::{Context, Result};
use toml;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandRegistry {
    pub commands: HashMap<String, String>,
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
            if let Some(template) = value.as_str() {
                commands.insert(key.clone(), template.to_string());
            }
        }

        Ok(CommandRegistry { commands })
    }

    pub fn get_command(&self, name: &str) -> Option<&String> {
        self.commands.get(name)
    }

    pub fn format_prompt(&self, name: &str, args: &str) -> Option<String> {
        self.get_command(name).map(|template| {
            template.replace("{args}", args)
        })
    }
}
