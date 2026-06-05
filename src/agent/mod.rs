//! The `agent` module orchestrates the core AI agent logic.
//!
//! This module includes agent construction, specialist tool management, 
//! MCP server integration, and reflection services.

pub mod agent;
pub mod mcp;
pub mod reflection;
pub mod specialists;

pub use agent::{
    AppConfig, ModelConfig, build_agent, get_compaction_config, load_config_sync,
    save_config_sync, load_model_with_fallback,
};
