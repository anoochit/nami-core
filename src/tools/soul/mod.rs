use crate::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::fs;

#[derive(Deserialize, JsonSchema)]
struct UpdateMemoryArgs {
    /// The new fact or preference to remember about the user.
    fact: String,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateSoulArgs {
    /// The new personality trait or instruction for the agent.
    trait_info: String,
}

/// Adds a new fact to the user's permanent memories (workspace/MEMORIES.md).
#[tool]
async fn update_user_memory(args: UpdateMemoryArgs) -> std::result::Result<Value, AdkError> {
    let workspace = get_workspace_dir().await?;
    let path = workspace.join("MEMORIES.md");
    
    let mut content = fs::read_to_string(&path)
        .await
        .unwrap_or_else(|_| "# MEMORIES\n".to_string());
    content.push_str(&format!("\n- {}", args.fact));

    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to update memories: {}", e)))?;

    Ok(json!({"status": "success", "message": "I'll remember that for you!"}))
}

/// Updates the agent's persona file (workspace/AGENT.md). Use this to 'evolve' the agent's personality.
#[tool]
async fn update_agent_soul(args: UpdateSoulArgs) -> std::result::Result<Value, AdkError> {
    let workspace = get_workspace_dir().await?;
    let path = workspace.join("AGENT.md");

    let mut content = fs::read_to_string(&path)
        .await
        .unwrap_or_else(|_| "# NAMI\n".to_string());
    content.push_str(&format!("\n\n## Evolution\n{}", args.trait_info));

    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to update soul: {}", e)))?;

    Ok(json!({"status": "success", "message": "My soul is evolving... thank you!"}))
}

pub fn soul_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(UpdateUserMemory), Arc::new(UpdateAgentSoul)]
}
