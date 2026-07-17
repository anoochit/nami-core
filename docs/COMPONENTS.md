# Component Reference: Core Modules & Directories

This document provides a comprehensive, detailed breakdown of all primary components, directories, and modules within the Nami framework.

## 🏃 Core Orchestration & Services

### 1. Agent Runner (`src/runner.rs`)

The `AgentRunner` is the main orchestration struct used across all interaction modes to execute agent tasks.

- **Persistence**: Integrates with `adk-session` and `adk-memory` to maintain conversational and semantic state across restarts.
- **Retry Strategy**: Implements an exponential backoff loop for transient errors (such as network or API rate limits, categorized in `src/utils/mod.rs`).
- **Event Compaction**: Automatically summarizes conversation history when context window limits are approached, using the `EventsCompactionConfig`.
- **Streaming**: Provides a clean interface for handling asynchronous event streams from the ADK runner.

### 2. The Agent Core (`src/agent/agent.rs`)

The `Agent` is the central brain and identity of the system, constructed via `build_agent()`.

- **Persona Loading**: Dynamically constructs the system prompt by reading:
  - `AGENT.md`: Core identity, tone, and behavioral rules.
  - `USER.md`: User profile and customized preferences.
  - `MEMORIES.md`: Synthesized long-term facts.
- **Skill Discovery**: Automatically loads "Skills" (standalone agent executable logic) from the `workspace/` directory using `with_skills_from_root`.
- **Specialists**: Manages a set of specialized sub-agents (e.g., Coder, Researcher) that can be invoked dynamically for specific workloads.

### 3. Shared Dependencies (`src/modes/startup.rs`)

The `Dependencies` struct holds shared service instances used globally throughout the application.

- **Session Service**: A SQLite-backed service for storing conversation history and agent state.
- **Memory Service**: A SQLite-backed vector/semantic memory service for long-term fact retrieval.
- **Memory Adapter**: Bridges the specific memory service to the general `adk-rust::Memory` trait used by tools.

### 4. Reflection Service (`src/agent/reflection.rs`)

A background service that acts as Nami's "Memory Architect."

- **Observation**: Periodically reads recent session logs.
- **Synthesis**: Uses a specific reflection model to extract new learnings, facts, and corrections.
- **Updates**: Automatically updates the `MEMORIES.md` file and the searchable memory database, ensuring Nami grows smarter over time.

### 5. Scheduler (`src/modes/scheduler.rs`)

A background loop that enables autonomous and scheduled agent actions.

- **Cron Support**: Executes tasks based on defined schedules.
- **Self-Correction**: Uses the agent's logic to determine if a scheduled task was successful or requires follow-up.

---

## 📂 Codebase Module Reference

Nami is structured into four primary modules under `src/`. Below is a detailed breakdown of each module's architecture and responsibilities.

### 🧠 1. Agent Module (`src/agent/`)

**Purpose**: Orchestrates the core AI agent logic, managing configuration, persona, tool registration, and the lifecycle of the agent builder. It serves as the primary gateway for defining Nami's personality and capabilities.

#### Architecture & Sub-files

- **`agent.rs`**: Core factory for agent construction, persona formatting, and instruction management.
- **`specialists.rs`**: Logic for managing and injecting specialist agent tools (e.g., coder, writer). Includes the `StreamSpecialistAgent` wrapper to parse and output streaming thoughts, tool invocation markers, and status indicators in real-time.
- **`mcp.rs`**: Handles loading and registration of Model Context Protocol (MCP) servers and their tools.
- **`reflection.rs`**: Background service that synthesizes conversational history into persistent insights.

#### Key Entry Points

- `create_agent(&AppConfig)`: Main factory function for building a configured agent, utilizing the central `create_core_tools` factory.
- `build_agent()`: Wrapper to load `config.toml` and initialize the agent setup process.
- `format_persona(...)`: Assembles the system prompt template.

#### Dependencies & Maintenance

- **Dependencies**: `adk-rust`, `adk-runner`, `crate::tools`, `crate::utils`
- **Maintenance**:
  - Built-in specialized agents are registered in `specialists.rs` and added to the `core_tools` list in `agent.rs`.
  - Arbitrary custom specialist agents can be configured dynamically under `[specialists.custom]` in the global `config.toml` (or `~/.nami/config.toml`) without modifying the source code. Custom agents automatically load custom LLM models/overrides, custom descriptions, and custom instructions (system prompts), registering themselves for delegation.
  - **Built-in Specialist Profiles**:
    * **`generalist`**: Handles high-volume batch tasks or repetitive data-processing pipelines.
    * **`coder`**: Expert in system design, debugging, and full-stack software development.
    * **`researcher`**: Specializes in information retrieval, documentation synthesis, and data research.
    * **`writer`**: Tailored for clear technical writing, specifications, and user documentation.
    * **`ralph`**: Autonomous persistence loop agent that doesn't stop until goals are achieved.
    * **`verifier`**: Review specialist that validates results against test schemas and filesystem structures.
    * **`designer`**: High-fidelity frontend developer specializing in responsive utility-first Tailwind CSS, custom aesthetic themes, gradients, and micro-interactions.
  - System instructions (`format_persona`) must remain concise to stay within token limits.

---

### 🖥️ 2. Interface Layer (`src/modes/`)

**Purpose**: Contains the entry-point implementations for the various ways Nami can be run (CLI, Telegram Bot, HTTP Server, etc.). Each mode acts as an interface layer, bridging user input to the core agent logic.

#### Architecture & Sub-files

- **`cli.rs`**: Interactive CLI interface with rich formatting and command handling.
- **`bot.rs`**: Telegram integration using the `teloxide` framework.
- **`line.rs`**: LINE Bot integration with webhook verification and messaging API support.
- **`serve.rs`**: HTTP REST API for external integrations.
- **`init.rs`**: Logic for bootstrapping the project configuration and database.
- **`api.rs`**: RESTful API endpoints for workspace and wiki access.
- **`run.rs`**: Non-interactive direct execution of the agent with a single prompt.
- **`upgrade.rs`**: Binary self-upgrade system that queries GitHub Releases for snapshot assets, checks hardware profiles, prompts the user, and conducts safe hot-swaps of active binaries.
- **`commands.rs`**: Core async state-machine and interceptor middleware that handles slash commands (like `/plan` and `/grill`) across CLI, WebUI, and Chatbots. Saves active grilling/planning states to the session database.

> [!NOTE]
> Quiet mode logic is implemented in `src/main.rs`. When running `run`, `eval`, or `workspace`, all verbose initialization/telemetry logs are suppressed to keep standard outputs clean and script-friendly.

#### Dependencies & Maintenance

- **Dependencies**: `axum`, `teloxide`, `tower-http`, `crate::agent`, `crate::runner`
- **Maintenance**:
  - When adding a new run mode, ensure the `Commands` enum in `src/main.rs` is updated to include the new variant and registered in the `main` loop.
  - Any shared UI helpers should be placed in `ui_utils.rs`.

---

### ⚙️ 3. Security & Sandboxing (`src/utils/`)

**Purpose**: Provides shared, low-level utilities and helper functions focused on system infrastructure, security enforcement, and path management.

#### Architecture & Sub-files

- **`mod.rs`**: Core workspace path management (`get_workspace_dir`, `sandbox`) and wiki directory helpers.
- **`ignore.rs`**: Implementation of the `.namiignore` policy system, ensuring security compliance for file-system tools.

#### Key Entry Points

- `get_workspace_dir()`: Retrieves the secure, sandboxed project workspace root.
- `sandbox(&str)`: Path validation utility that prevents directory traversal and enforces ignore policies.
- `get_wiki_dir()`: Standardized path getter for wiki-related operations.

#### Maintenance Notes

- Keep this module lean and focused on project-wide infrastructure.
- Any new security policies or path-sanitization rules must be implemented here to ensure global adherence across all tools.

---

### 🛠️ 4. Tools Module (`src/tools/`)

**Purpose**: Defines all executable capabilities available to the Nami agent. Each sub-module represents a specific domain-driven toolset, allowing the agent to perform real-world tasks like file I/O, web searching, wiki management, and system monitoring.

#### Built-in Toolsets

- **`analyze_media/`**: Unified multimodal tool designed to natively parse and reason over non-text resources (such as PNG/JPEG, MP3/WAV, MP4/MOV, and PDFs) by extracting inline raw byte vectors directly into multimodal-capable model APIs.
- **`audio_generator/` / `video_generator/`**: High-fidelity tools enabling generative text-to-speech/sound effects and image-to-video production with detailed voice, camera, speed, and motion controls.
- **`current_datetime/`**: Provides the current date, time, and timezone offset information using system clock calculations.
- **`evolution/`**: Periodically updates internal `MEMORIES.md` and `AGENT.md` instructions using global sandboxed directory paths via `get_nami_dir()`.
- **`filesystem/`**: Provides sandboxed file system operations (read, write, list, delete) safely within the workspace with cleaned execution response metrics.
- **`image_generator/`**: Native AI image generation capabilities with exposed JSON parameter schema and absolute local output file path translations.
- **`invoke_agent/`**: Invokes a single specialist agent (e.g., coder, researcher, writer) by name with a given prompt to delegate tasks.
- **`memory/`**: Vector-searchable long-term memory operations (`add_memory`, `recall_memory`) backed by SQLite.
- **`parallel_tasks/`**: Orchestration logic to run multiple specialist agents concurrently.
- **`plan/`**: Integrated Autonomous Planner-Executor-Verifier toolset. Supports structured implementation planning (`plan_create` with custom pre-synthesized steps, `plan_show`, `plan_list`, `plan_delete`, `plan_update`), interactive plan alignment (`PlanGrill` helper for CLI and conversational Q&A-driven planning), and complete autonomous plan execution (`plan_execute`) with self-healing, critic verification, and dynamic replanning.
- **`scheduler/`**: Background task scheduler running operations based on cron expressions.
- **`search/`**: Web search integration using external APIs (e.g., Serper.dev) for real-time information retrieval.
- **`shell/`**: Executes shell commands with structured validation controls, including customizable `security_level` limits, `allowed_commands` whitelists, `blocked_commands` restrictions, and path traversal validation checks.
- **`soul/`**: Tools for managing and updating the agent's internal persona/soul and user memory.
- **`supervised_delegate/`**: Concurrent multi-agent supervisor orchestrator. Formulates complex goals into a Directed Acyclic Graph (DAG) of specialized subtasks, executes independent DAG branches concurrently via `tokio::spawn`, runs an autonomous QA self-correction loop, and generates a master synthesized final report.
- **`system_status/`**: Monitors and reports system health (CPU, memory, general performance metrics, and network latency checks).
- **`todo/`**: A simple task/TODO list manager that persists items to `todos.json` in the workspace.
- **`weather/`**: Queries real-time weather forecasts and conditions with multi-day metrics.
- **`web_fetch/`**: Fetches content from arbitrary URLs, automatically converting fetched HTML to markdown utilizing the `html2md` module to improve LLM reading quality and reduce downstream token usage.
- **`wiki/`**: Knowledge management system tools for Obsidian-style Markdown manipulation, backlinks, and autogenerated graphs.

#### Key Entry Points & Maintenance

- **Entry Point**: `mod.rs` exports the centralized `create_core_tools` factory that parses `AppConfig` and `ToolFactoryConfig` to dynamically enable or disable tool category domains and perform dependency injections.
- **Maintenance**:
  - Tools must be registered in the respective module's subfolder and wired inside `create_core_tools` in `src/tools/mod.rs`.
  - Always wrap filesystem paths using `crate::utils::sandbox` to prevent security risks.
