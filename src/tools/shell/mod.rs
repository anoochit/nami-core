use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::process::Command;

#[derive(Deserialize, JsonSchema)]
struct ShellArgs {
    /// Command to execute
    command: String,
}

/// Executes allowed system commands safely.
#[tool]
async fn execute_shell(args: ShellArgs) -> std::result::Result<Value, AdkError> {
    // Basic security: only allow specific commands
    let allowed_commands = ["git", "ls", "grep"];
    if !allowed_commands
        .iter()
        .any(|&cmd| args.command.starts_with(cmd))
    {
        return Err(AdkError::tool(format!(
            "Command not allowed: {}",
            args.command
        )));
    }

    let output = Command::new("powershell")
        .arg("-Command")
        .arg(&args.command)
        .output()
        .await
        .map_err(|e| AdkError::tool(format!("Execution failed: {}", e)))?;

    if output.status.success() {
        Ok(json!({"stdout": String::from_utf8_lossy(&output.stdout)}))
    } else {
        Err(AdkError::tool(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn shell_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ExecuteShell)]
}
