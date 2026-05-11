//! The `agent` module orchestrates the core AI agent logic.
//!
//! This module includes agent construction, specialist tool management, 
//! MCP server integration, and reflection services.

pub mod agent;
pub mod mcp;
pub mod reflection;
pub mod specialists;

pub use agent::{build_agent, get_compaction_config};
