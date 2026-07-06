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

/// Helper function to validate complex, potentially chained shell commands.
fn validate_command(command: &str) -> std::result::Result<(), String> {
    // Whitelist of allowed base commands
    let allowed_executables = [
        "git", "ls", "grep", "echo", "cargo", "nami", "dir", "type", "cat", "pwd"
    ];

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

        if !allowed_executables.contains(&exe_clean.as_str()) {
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

/// Executes allowed system commands safely with strict command-chaining validation.
#[tool]
async fn execute_shell(args: ShellArgs) -> std::result::Result<Value, AdkError> {
    if let Err(err_msg) = validate_command(&args.command) {
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

pub fn shell_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ExecuteShell)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_commands() {
        assert!(validate_command("git status").is_ok());
        assert!(validate_command("cargo test --all").is_ok());
        assert!(validate_command("git add . && git commit -m 'test'").is_ok());
        assert!(validate_command("git log | grep fix").is_ok());
    }

    #[test]
    fn test_invalid_commands() {
        assert!(validate_command("rm -rf /").is_err());
        assert!(validate_command("git status && rm -rf .").is_err());
        assert!(validate_command("cargo build; format C:").is_err());
        assert!(validate_command("git status; $(rm -rf /)").is_err());
    }
}
