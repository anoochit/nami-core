use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::AdkError;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::process::Command;
use std::path::PathBuf;

#[derive(Deserialize, JsonSchema)]
struct ShellArgs {
    /// Command to execute
    command: String,
}

#[derive(Clone, Debug, Default)]
pub struct ShellConfig {
    pub allowed_commands: Option<Vec<String>>,
    pub blocked_commands: Option<Vec<String>>,
    pub security_level: Option<String>, // "strict" (default) or "permissive"
    pub sanitize_environment: Option<bool>,
}

pub struct ExecuteShell {
    config: ShellConfig,
    allowed_executables: Vec<String>,
}

impl ExecuteShell {
    pub fn new(config: ShellConfig) -> Self {
        let mut allowed = vec![
            "git".to_string(), "ls".to_string(), "grep".to_string(), "echo".to_string(),
            "cargo".to_string(), "nami".to_string(), "dir".to_string(), "type".to_string(),
            "cat".to_string(), "pwd".to_string(), "df".to_string(), "ip".to_string(),
            "uname".to_string(), "python3".to_string(), "node".to_string(), "npm".to_string(),
            "docker".to_string()
        ];
        if let Some(ref cfg_allowed) = config.allowed_commands {
            for cmd in cfg_allowed {
                let cmd_clean = cmd.trim().to_lowercase();
                if !cmd_clean.is_empty() && !allowed.contains(&cmd_clean) {
                    allowed.push(cmd_clean);
                }
            }
        }
        Self {
            config,
            allowed_executables: allowed,
        }
    }

    /// Verifies if any argument in the command contains unsafe path traversals (e.g. escaping the workspace)
    fn check_path_traversal(&self, command: &str) -> std::result::Result<(), String> {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let nami_dir = home.join(".nami");
        let agents_dir = home.join(".agents");

        // Split command line by simple whitespace to inspect arguments for paths
        for token in command.split_whitespace() {
            // Normalize path separator for checks
            let token_normalized = token.replace('\\', "/");
            
            // Check if token references parent directory traversal
            if token_normalized.contains("../") || token_normalized.contains("..") {
                // If it contains "..", construct resolved path and check if it escapes workspace
                let mut base_path = root.clone();
                for part in token_normalized.split('/') {
                    if part == ".." {
                        base_path.pop();
                    } else if part != "." && !part.is_empty() {
                        base_path.push(part);
                    }
                }
                
                if !base_path.starts_with(&root) && !base_path.starts_with(&nami_dir) && !base_path.starts_with(&agents_dir) {
                    return Err(format!(
                        "Security Error: Argument '{}' attempts parent directory traversal outside permitted directories.",
                        token
                    ));
                }
            }

            // Check if absolute path escapes workspace
            if token_normalized.starts_with('/') {
                let abs_path = PathBuf::from(token);
                if !abs_path.starts_with(&root) && !abs_path.starts_with(&nami_dir) && !abs_path.starts_with(&agents_dir) {
                    return Err(format!(
                        "Security Error: Absolute path argument '{}' is outside the sandboxed workspace.",
                        token
                    ));
                }
            }
        }
        Ok(())
    }

    /// Helper function to validate complex, potentially chained shell commands.
    fn validate_command(&self, command: &str) -> std::result::Result<(), String> {
        let is_strict = self.config.security_level.as_deref().unwrap_or("strict").to_lowercase() != "permissive";
        let blocked_commands = self.config.blocked_commands.as_ref();

        // Prevent subshell execution / injection tricks
        if command.contains('`') || command.contains("$(") {
            return Err("Subshell executions (backticks or $()) are strictly forbidden for security reasons.".to_string());
        }

        // Apply path traversal argument verification
        self.check_path_traversal(command)?;

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

            // Check blocked list if provided (always applies)
            if let Some(blocked) = blocked_commands {
                if blocked.iter().any(|b| b.to_lowercase() == exe_clean) {
                    return Err(format!("Command executable '{}' is explicitly blocked by policy.", exe));
                }
            }

            // Strict mode whitelist check
            if is_strict && !self.allowed_executables.contains(&exe_clean) {
                return Err(format!(
                    "Command executable '{}' is not in the security whitelist under Strict mode. Allowed: {:?}",
                    exe, self.allowed_executables
                ));
            }

            Ok(())
        };

        // Recursively split the command by all known logical separators
        let separators = ["&&", "||", ";", "|", "\n", "\r"];
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
        "Executes allowed system commands safely with strict command-chaining validation and environment isolation."
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

        let mut cmd = Command::new(shell);
        cmd.arg(flag).arg(&args.command);

        // Sanitize environment variables if enabled (default true)
        if self.config.sanitize_environment.unwrap_or(true) {
            cmd.env_clear();
            // Retain path and essential system variables
            if let Ok(path) = std::env::var("PATH") {
                cmd.env("PATH", path);
            }
            if let Ok(home) = std::env::var("HOME") {
                cmd.env("HOME", home);
            }
            if let Ok(user) = std::env::var("USER") {
                cmd.env("USER", user);
            }
            if let Ok(term) = std::env::var("TERM") {
                cmd.env("TERM", term);
            }
            // Retain workspace context variables
            if let Ok(nami_ws) = std::env::var("NAMI_WORKSPACE") {
                cmd.env("NAMI_WORKSPACE", nami_ws);
            }
        }

        let output = cmd
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

pub fn shell_tools(config: Option<ShellConfig>) -> Vec<Arc<dyn Tool>> {
    let cfg = config.unwrap_or_default();
    vec![Arc::new(ExecuteShell::new(cfg))]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_tool(level: &str) -> ExecuteShell {
        ExecuteShell::new(ShellConfig {
            allowed_commands: Some(vec!["git".to_string(), "cargo".to_string(), "cat".to_string()]),
            blocked_commands: Some(vec!["rm".to_string(), "dd".to_string()]),
            security_level: Some(level.to_string()),
            sanitize_environment: Some(true),
        })
    }

    #[test]
    fn test_valid_commands() {
        let tool = get_test_tool("strict");
        assert!(tool.validate_command("git status").is_ok());
        assert!(tool.validate_command("cargo test --all").is_ok());
    }

    #[test]
    fn test_blocked_commands() {
        let tool = get_test_tool("permissive");
        assert!(tool.validate_command("rm -rf /").is_err());
    }

    #[test]
    fn test_path_traversal() {
        let tool = get_test_tool("permissive");
        assert!(tool.validate_command("cat ../../../etc/passwd").is_err());
        assert!(tool.validate_command("cat /etc/passwd").is_err());
    }
}
