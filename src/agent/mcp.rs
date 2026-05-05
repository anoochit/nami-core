use crate::utils;
use adk_rust::prelude::*;
use adk_tool::mcp::McpServerManager;
use adk_tool::toolset::PrefixedToolset;
use std::sync::Arc;

/// Loads MCP tools from `mcp.json` if it exists and attaches them to the agent builder.
///
/// It checks the workspace directory first, then the current directory. If a configuration
/// file is found, it initializes an `McpServerManager`, attempts to start all defined
/// MCP servers, logs any failures, and registers the manager as a toolset with the agent.
pub async fn load_mcp_tools(mut builder: LlmAgentBuilder) -> anyhow::Result<LlmAgentBuilder> {
    // Determine the path to the configuration file
    let workspace_root: std::path::PathBuf = utils::get_workspace_dir().await?;
    let workspace_mcp = workspace_root.join("mcp.json");

    let mcp_config_path = if workspace_mcp.exists() {
        Some(workspace_mcp)
    } else if std::path::Path::new("mcp.json").exists() {
        Some(std::path::PathBuf::from("mcp.json"))
    } else {
        None
    };

    if let Some(path) = mcp_config_path {
        // Initialize the manager and load servers
        let mcp_manager = McpServerManager::from_json_file(path.to_str().unwrap_or("mcp.json"))?;
        let mcp_manager = Arc::new(mcp_manager);

        // Start all servers and handle potential failures per server
        let results = mcp_manager.start_all().await;
        for (name, res) in results {
            if let Err(e) = res {
                log::error!("Failed to start MCP server '{}': {}", name, e);
            } else {
                log::info!("Started MCP server '{}'", name);
            }
        }

        // Attach the started manager to the agent as a toolset with a prefix to avoid collisions
        builder = builder.toolset(Arc::new(PrefixedToolset::new(mcp_manager, "mcp")));
    }

    Ok(builder)
}
