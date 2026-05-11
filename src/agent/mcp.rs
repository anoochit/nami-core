use crate::utils;
use adk_rust::ReadonlyContext;
use adk_rust::prelude::*;
use adk_tool::mcp::{
    AutoDeclineElicitationHandler, McpHttpClientBuilder, McpServerConfig, McpServerManager,
};
use adk_tool::toolset::{MergedToolset, PrefixedToolset};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Holds configuration for multiple MCP servers.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CombinedMcpConfig {
    /// Mapping of server name to server configuration.
    mcp_servers: HashMap<String, McpServerEntry>,
}

/// Represents the transport configuration for an MCP server.
#[derive(Deserialize)]
#[serde(untagged)]
enum McpServerEntry {
    /// Remote server accessed over HTTP.
    Http { 
        /// Base URL for the MCP server.
        url: String 
    },
    /// Local server accessed via Stdio.
    Stdio(McpServerConfig),
}

/// A tool wrapper that sanitizes JSON schemas by removing all extension fields (keys starting with `x-`).
///
/// This is necessary because Gemini's API rejects any unknown fields in tool declarations.
struct SanitizedTool {
    inner: Arc<dyn Tool>,
}

impl SanitizedTool {
    fn new(inner: Arc<dyn Tool>) -> Self {
        Self { inner }
    }

    /// Recursively removes all keys starting with `x-` from a JSON value.
    fn sanitize(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.retain(|k, _| !k.starts_with("x-"));
                for v in map.values_mut() {
                    Self::sanitize(v);
                }
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    Self::sanitize(v);
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Tool for SanitizedTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn enhanced_description(&self) -> String {
        self.inner.enhanced_description()
    }

    fn is_long_running(&self) -> bool {
        self.inner.is_long_running()
    }

    fn parameters_schema(&self) -> Option<Value> {
        let mut schema = self.inner.parameters_schema()?;
        Self::sanitize(&mut schema);
        Some(schema)
    }

    fn response_schema(&self) -> Option<Value> {
        let mut schema = self.inner.response_schema()?;
        Self::sanitize(&mut schema);
        Some(schema)
    }

    fn required_scopes(&self) -> &[&str] {
        self.inner.required_scopes()
    }

    fn declaration(&self) -> Value {
        let mut decl = self.inner.declaration();
        Self::sanitize(&mut decl);
        decl
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        self.inner.execute(ctx, args).await
    }
}

/// A toolset wrapper that sanitizes all its tools.
struct SanitizedToolset {
    inner: Arc<dyn Toolset>,
}

impl SanitizedToolset {
    fn new(inner: Arc<dyn Toolset>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl Toolset for SanitizedToolset {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn tools(&self, ctx: Arc<dyn ReadonlyContext>) -> Result<Vec<Arc<dyn Tool>>> {
        let tools = self.inner.tools(ctx).await?;
        Ok(tools
            .into_iter()
            .map(|t| Arc::new(SanitizedTool::new(t)) as Arc<dyn Tool>)
            .collect())
    }
}

/// Loads MCP tools from `mcp.json` if it exists and attaches them to the agent builder.
///
/// It supports both `stdio` (local processes) and `http` (remote streamable HTTP) transports.
/// It checks the workspace directory first, then the current directory.
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
        let content = std::fs::read_to_string(&path)?;
        let config: CombinedMcpConfig = serde_json::from_str(&content)?;

        let mut stdio_configs = HashMap::new();
        let mut http_toolsets = Vec::new();

        for (name, entry) in config.mcp_servers {
            match entry {
                McpServerEntry::Http { url } => {
                    log::info!("Connecting to remote MCP server '{}' at {}", name, url);
                    let mut client_builder = McpHttpClientBuilder::new(url);

                    // Use auto-decline elicitation handler for consistency with McpServerManager
                    client_builder = client_builder
                        .with_elicitation_handler(Arc::new(AutoDeclineElicitationHandler));

                    match client_builder.connect().await {
                        Ok(toolset) => {
                            // Wrap with name prefix for consistency
                            http_toolsets
                                .push(Arc::new(PrefixedToolset::new(Arc::new(toolset), &name)));
                        }
                        Err(e) => {
                            log::error!("Failed to connect to remote MCP server '{}': {}", name, e);
                        }
                    }
                }
                McpServerEntry::Stdio(stdio_config) => {
                    stdio_configs.insert(name, stdio_config);
                }
            }
        }

        let mut all_toolsets: Vec<Arc<dyn Toolset>> = Vec::new();

        // Initialize Stdio manager if there are any stdio servers
        if !stdio_configs.is_empty() {
            let mcp_manager = McpServerManager::new(stdio_configs);
            let results = mcp_manager.start_all().await;
            for (name, res) in results {
                if let Err(e) = res {
                    log::error!("Failed to start MCP server '{}': {}", name, e);
                } else {
                    log::info!("Started MCP server '{}'", name);
                }
            }
            all_toolsets.push(Arc::new(mcp_manager));
        }

        // Add HTTP toolsets
        for ts in http_toolsets {
            all_toolsets.push(ts as Arc<dyn Toolset>);
        }

        if !all_toolsets.is_empty() {
            // Merge all toolsets and apply the global "mcp" prefix
            let merged = MergedToolset::new("mcp_merged", all_toolsets);
            // Sanitize the merged toolset to remove Gemini-incompatible fields
            let sanitized = SanitizedToolset::new(Arc::new(merged));
            builder = builder.toolset(Arc::new(PrefixedToolset::new(Arc::new(sanitized), "mcp")));
        }
    }

    Ok(builder)
}
