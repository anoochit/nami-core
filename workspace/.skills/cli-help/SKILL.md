---
name: cli-help
description: Reference guide for Nami CLI commands, flags, and usage patterns.
---

# CLI Help (Nami)

This skill provides a centralized reference for interacting with the **Nami CLI**.

Use `nami help` at any time to display this information in the terminal.

---

## Available Commands

### Core Commands
- `init`  
  Initialize project configuration.

- `serve`  
  Start the API server.

- `cli`  
  Launch the interactive TUI interface.

---

### Bot Integration
- `bot`  
  Start the Telegram bot service.

---

### Prompt Execution
- `run "<prompt>"`  
  Execute a prompt directly from the CLI.

---

### Help
- `help`  
  Display usage instructions and available commands.

---

## Usage Notes

- Commands can be executed from any directory with a valid Nami setup.
- Prompts passed via CLI are executed in the current workspace context.
- Interactive mode (`cli`) is recommended for exploratory workflows.

---

## Troubleshooting

- **Command not found**
  - Ensure Nami CLI is installed.
  - Verify that the binary is available in your system `PATH`.

- **Execution errors**
  - Check environment variables and configuration files.
  - Ensure the workspace is properly initialized (`nami init`).

- **Bot not starting**
  - Verify required credentials (e.g., Telegram token).
  - Check network connectivity.

---

## When to Use This Skill

Use this skill when:
- You need to recall CLI commands or syntax.
- You want to guide a user on how to use Nami CLI.
- You are constructing or validating CLI-based workflows.