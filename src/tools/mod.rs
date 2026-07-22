//! The `tools` module defines all executable capabilities available to the Nami agent.
//!
//! Each sub-module represents a specific domain-driven toolset, allowing the agent to perform real-world tasks.

pub mod current_datetime;
pub mod invoke_agent;
pub mod filesystem;
pub mod image_generator;
pub mod audio_generator;
pub mod video_generator;
pub mod memory;
pub mod parallel_tasks;
pub mod scheduler;
pub mod search;
pub mod shell;
pub mod soul;
pub mod todo;
pub mod web_fetch;
pub mod wiki;
pub mod evolution;
pub mod supervised_delegate;

use std::sync::Arc;
use adk_rust::prelude::*;

/// Configuration for dynamic and modular tool generation.
pub struct ToolFactoryConfig {
    pub model: Arc<dyn Llm>,
    pub model_name: String,
    pub image_model: Option<Arc<dyn Llm>>,
    pub audio_model: Option<Arc<dyn Llm>>,
    pub video_model: Option<Arc<dyn Llm>>,
    pub shell_config: Option<shell::ShellConfig>,
    pub enabled_categories: Option<Vec<String>>,
}

/// Dynamically discovers and instantiates enabled core tools for the agent.
pub fn create_core_tools(config: ToolFactoryConfig) -> Vec<Arc<dyn Tool>> {
    let mut tools = Vec::new();

    let is_enabled = |name: &str| -> bool {
        if let Some(ref enabled) = config.enabled_categories {
            enabled.iter().any(|cat| cat.to_lowercase() == name.to_lowercase())
        } else {
            true
        }
    };

    if is_enabled("datetime") {
        tools.extend(current_datetime::datetime_tools());
    }
    if is_enabled("filesystem") {
        tools.extend(filesystem::filesystem_tools(config.model.clone(), config.model_name.clone()));
    }
    if is_enabled("image_generator") {
        tools.extend(image_generator::image_generator_tools(config.image_model));
    }
    if is_enabled("audio_generator") {
        tools.extend(audio_generator::audio_generator_tools(config.audio_model));
    }
    if is_enabled("video_generator") {
        tools.extend(video_generator::video_generator_tools(config.video_model));
    }
    if is_enabled("memory") {
        tools.extend(memory::memory_tools());
    }
    if is_enabled("scheduler") {
        tools.extend(scheduler::scheduler_tools());
    }
    if is_enabled("search") {
        tools.extend(search::search_tools());
    }
    if is_enabled("shell") {
        tools.extend(shell::shell_tools(config.shell_config));
    }
    if is_enabled("soul") {
        tools.extend(soul::soul_tools());
    }
    if is_enabled("todo") {
        tools.extend(todo::todo_tools());
    }
    if is_enabled("web_fetch") {
        tools.extend(web_fetch::web_fetch_tools());
    }
    if is_enabled("wiki") {
        tools.extend(wiki::wiki_tools());
    }
    if is_enabled("evolution") {
        tools.extend(evolution::evolution_tools(config.model.clone()));
    }

    // LoadArtifactsTool allows the agent to dynamically load versioned binary files
    tools.push(Arc::new(adk_tool::LoadArtifactsTool::new()));

    tools
}


