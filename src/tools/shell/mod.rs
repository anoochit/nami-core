use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::AdkError;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::process::Command;

#[derive(Deserialize, JsonSchema)]
struct ShellArgs {
    /// Command to execute
    command: String,
}

pub struct ExecuteShell {
    allowed_executables: Vec<String>,
}

impl ExecuteShell {
    pub fn new(allowed_executables: Vec<String>) -> Self {
        Self { allowed_executables }
    }

    /// Helper function to validate complex, potentially chained shell commands.
    fn validate_command(&self, command: &str) -> std::result::Result<(), String> {
        let allowed_executables = &self.allowed_executables;
        // Logical separators for chained or piped commands
        let separators = ["&&", "||", ";", "|", "\n", "\r"];

        // Prevent subshell execution / injection tricks
        if command.contains('`') || command.contains("$(") {
            return Err("Subshell executions (backticks or $()) are strictly forbidden for security reasons.".to_string());
        }

        // Helper to validate an individual segment
        let check_segment = |segment: &str| -> std::result::Result<(), String> {
            let segment = segment.trim();
            if segment.is_empty() {
                return Ok(());
            }

            // Extract the executable (first word of the segment)
            let exe = segment
                .split_whitespace()
                .next()
                .ok_or_else(|| "Empty command segment".to_string())?;

            // Normalize Windows-style executable extensions
            let exe_clean = exe
                .trim_end_matches(".exe")
                .trim_end_matches(".cmd")
                .trim_end_matches(".bat")
                .to_lowercase();

            if !allowed_executables.contains(&exe_clean) {
                return Err(format!(
                    "Command executable '{}' is not in the security whitelist. Allowed: {:?}",
                    exe, allowed_executables
                ));
            }

            Ok(())
        };

        // Recursively split the command by all known logical separators
        let mut segments = vec![command.to_string()];
        for sep in &separators {
            let mut next_segments = Vec::new();
            for s in segments {
                for part in s.split(sep) {
                    next_segments.push(part.to_string());
                }
            }
            segments = next_segments;
        }

        // Validate every individual segment
        for segment in segments {
            check_segment(&segment)?;
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for ExecuteShell {
    fn name(&self) -> &str {
        "execute_shell"
    }

    fn description(&self) -> &str {
        "Executes allowed system commands safely with strict command-chaining validation."
    }

    async fn execute(
        &self,
        _context: Arc<dyn adk_rust::ToolContext>,
        args: Value,
    ) -> std::result::Result<Value, AdkError> {
        let args: ShellArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        if let Err(err_msg) = self.validate_command(&args.command) {
            return Err(AdkError::tool(format!("Security Violation: {}", err_msg)));
        }

        // Determine the shell interpreter based on the platform
        let shell = if cfg!(target_os = "windows") {
            "powershell"
        } else {
            "sh"
        };

        let flag = if cfg!(target_os = "windows") {
            "-Command"
        } else {
            "-c"
        };

        let output = Command::new(shell)
            .arg(flag)
            .arg(&args.command)
            .output()
            .await
            .map_err(|e| AdkError::tool(format!("Execution failed: {}", e)))?;

        if output.status.success() {
            Ok(json!({
                "stdout": String::from_utf8_lossy(&output.stdout),
                "status": "success"
            }))
        } else {
            Err(AdkError::tool(format!(
                "Command failed with exit code {}.\nStderr: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }
}

pub fn shell_tools(allowed_commands: Option<Vec<String>>) -> Vec<Arc<dyn Tool>> {
    let mut allowed = vec![
        "git".to_string(), "ls".to_string(), "grep".to_string(), "echo".to_string(),
        "cargo".to_string(), "nami".to_string(), "dir".to_string(), "type".to_string(),
        "cat".to_string(), "pwd".to_string(), "df".to_string(), "ip".to_string(),
        "uname".to_string(), "python3".to_string(), "node".to_string(), "npm".to_string(),
        "docker".to_string()
    ];
    if let Some(cfg_allowed) = allowed_commands {
        for cmd in cfg_allowed {
            let cmd_clean = cmd.trim().to_lowercase();
            if !cmd_clean.is_empty() && !allowed.contains(&cmd_clean) {
                allowed.push(cmd_clean);
            }
        }
    }
    vec![Arc::new(ExecuteShell::new(allowed))]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_tool() -> ExecuteShell {
        ExecuteShell::new(vec![
            "git".to_string(), "ls".to_string(), "grep".to_string(), "echo".to_string(),
            "cargo".to_string(), "nami".to_string(), "dir".to_string(), "type".to_string(),
            "cat".to_string(), "pwd".to_string(), "df".to_string(), "ip".to_string(),
            "uname".to_string(), "python3".to_string(), "node".to_string(), "npm".to_string(),
            "docker".to_string()
        ])
    }

    #[test]
    fn test_valid_commands() {
        let tool = get_test_tool();
        assert!(tool.validate_command("git status").is_ok());
        assert!(tool.validate_command("cargo test --all").is_ok());
        assert!(tool.validate_command("git add . && git commit -m 'test'").is_ok());
        assert!(tool.validate_command("git log | grep fix").is_ok());
    }

    #[test]
    fn test_invalid_commands() {
        let tool = get_test_tool();
        assert!(tool.validate_command("rm -rf /").is_err());
        assert!(tool.validate_command("git status && rm -rf .").is_err());
        assert!(tool.validate_command("cargo build; format C:").is_err());
        assert!(tool.validate_command("git status; $(rm -rf /)").is_err());
    }
}
