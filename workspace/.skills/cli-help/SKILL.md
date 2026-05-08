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
| `help` | Display basic command-line usage instructions. |

---

## Interactive CLI Mode (`nami cli`)

Once inside the interactive CLI, you can use the following **Slash Commands** for specialized functionality:

### 🧠 Task Management
- `/plan <goal>`  
  Initializes a structured task with specific steps and state tracking.
  *Example:* `/plan Build a React portfolio`

- `/tasks`  
  Lists all active, in-progress, or blocked tasks.

- `/status`  
  Displays the current agent status and system telemetry.

### ⚡ Automation & Delegation
- `/parallel "<task 1>" "<task 2>" ...`  
  Orchestrates multiple specialized agents (`coder`, `researcher`, `writer`) to perform tasks simultaneously.
  *Example:* `/parallel "Write Rust code" "Research async traits"`

- `/goal <goal> | <stop condition>`  
  Triggers the **Ralph Wiggum** loop. The agent will autonomously retry and pivot until the stop condition is met (max 5 iterations).
  *Example:* `/goal "Find AI news" | "Summary is written to news.md"`

- `/schedule <goal> | <cron expression>`  
  Registers a persistent background task using standard cron syntax.
  *Example:* `/schedule "Backup workspace" | "0 0 * * * *"` (Runs every hour)

### 📂 Knowledge & Memory
- `/wiki <query>`  
  Performs a full-text search across your Obsidian-style Markdown vault.
- `/memo <fact>`  
  Explicitly saves a personal fact or preference to your `MEMORIES.md`.
- `@<file_path>`  
  Type `@` followed by a path (with tab-completion) to inject file contents into your prompt.

### 🛠 System
- `/new`  
  Starts a fresh session ID while maintaining persistent memory.
- `/clear`  
  Clears the terminal screen and re-renders the banner.
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
  - Verify your `.env` contains valid `GOOGLE_API_KEY` or `OPENAI_API_KEY`.

- **Access Denied**
  - Files outside the `workspace/` or matched by `.namiignore` are restricted for safety.

- **Background Tasks**
  - Scheduled tasks run quietly in the background of the `cli` mode. Check logs if a task fails to trigger.

---

## When to Use This Skill

Use this skill when:
- You need to recall specific command syntax or cron formats.
- You want to guide a user on how to use Nami's advanced features like Parallel or Goal loops.
- You are optimizing your CLI-based AI workflow.