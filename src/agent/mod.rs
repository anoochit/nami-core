//! Orchestrates the core AI agent logic including construction, specialists, MCP, and reflection.

pub mod agent;
pub mod config;
pub mod mcp;
pub mod reflection;
pub mod specialists;

pub use agent::{
    build_agent, get_compaction_config, get_intra_compaction_config,
};
pub use config::{
    AppConfig, ModelConfig, load_config_sync, save_config_sync, load_model_with_fallback,
};
