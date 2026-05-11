---
name: cli-help
description: Reference guide for Nami CLI commands, flags, and usage patterns.
---

# CLI Help (Nami)

This skill provides a centralized reference for interacting with the **Nami CLI**.

Use `nami help` at any time to display this information in the terminal.

---

## Core CLI Commands

| Command | Description |
| :--- | :--- |
| `init` | Initialize project configuration, persona files, and database. |
| `cli` | Launch the **Interactive TUI** (Recommended). |
| `run "<prompt>"` | Execute a single prompt directly from the terminal. |
| `bot` | Start the Telegram bot service. |
| `serve` | Start the HTTP API server. |
| `browse` | Start server with embedded WebUI. |
| `help` | Display basic command-line usage instructions. |

---

## Interactive CLI Mode (`nami cli`)

Once inside the interactive CLI, you can use the following **Slash Commands** for specialized functionality:

### 🧠 Task Management
- `/plan <goal>`  
  Initializes a structured task with specific steps and state tracking.
- `/tasks`  
  Lists all active, in-progress, or blocked tasks.
- `/status`  
  Displays the current agent status and system telemetry.

### ⚡ Automation & Delegation
- `/parallel "<task 1>" "<task 2>" ...`  
  Orchestrates multiple specialized agents to perform tasks simultaneously.
- `/goal <goal> | <stop condition>`  
  Triggers the autonomous "Ralph Wiggum" loop agent, which persists through multiple iterations to achieve complex goals.
- `/schedule <goal> | <cron expression>`  
  Registers a persistent background task using standard cron syntax.

### 📂 Knowledge & Memory
- `@<file_path>`  
  Reference files from the `workspace/` directly in the prompt using tab-completion.
- `/wiki <query>`  
  Searches your Obsidian-style Markdown vault.

### 🛠 System
- `/new`  
  Resets the current session while maintaining persistent memory.
- `/exit` or `/quit`  
  Safely closes the CLI session.
- `/?`  
  Displays the in-CLI help menu.

---

## Usage Patterns

### Direct Command Execution
Use the `run` command for simple, one-off tasks:
```bash
nami run "Summarize my latest wiki notes"
```

### Complex Multi-Tasking
Leverage the parallel orchestrator to speed up development:
```bash
nami cli
You > /parallel "Fix the unit tests in src/modes/" "Update the CHANGELOG.md"
```

### Autonomous Research
Use the goal loop for tasks that require multiple steps:
```bash
You > /goal "Research the best Rust HTTP libraries and create a comparison table in research.md" | "A file named research.md exists"
```

---

## Troubleshooting

- **Configuration Issues**
  - Run `nami init` to regenerate missing config files.
  - Verify your `.env` contains valid API keys.

- **Access Denied**
  - Files outside the `workspace/` or matched by `.namiignore` are restricted for safety.

---

## When to Use This Skill

Use this skill when:
- You need to recall specific command syntax or slash commands.
- You want to guide a user on how to use Nami's advanced features like Parallel, Goal loops, or Task management.
- You are optimizing your CLI-based AI workflow.