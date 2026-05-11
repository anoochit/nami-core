# Changelog

All notable changes to this project will be documented in this file.

## [0.9.7] - 2026-05-11

### Added
- **Book Reorganization**: Restructured the Nami documentation book into three thematic parts (Core, Action, Agency) and synchronized both English and Thai versions.
- **New Book Chapter**: Added "Chapter 8: The Digital Swiss Army Knife" to both language versions, covering utility tools like Todo, Datetime, and System Status.
- **Module Documentation**: Added comprehensive `README.md` files for all internal tool modules (`src/tools/*`) and modes (`src/modes/`) to improve codebase discoverability and maintenance.
- **Enhanced Exec Command**: Improved the `exec_command` tool with `stdin` support, enabling better cross-platform data passing for external scripts.

### Changed
- **Metadata Standardization**: Systematically updated all book chapter files with standard YAML frontmatter and consistent numbering.

### Removed
- **Imagen Tool**: Removed the AI image generation tool and its associated Python scripts from the core toolset and workspace skills.

## [0.9.6] - 2026-05-11

### Added

- **Workspace & Wiki API**: Added new REST endpoints (`/api/workspace/files` and `/api/wiki/pages`) to allow WebUI and external tools to access and read workspace files and wiki content safely.
- **Serve Mode API Support**: Integrated the custom API router into both `serve` and `browse` modes by manually managing the Axum server startup.
- **Sandbox Security**: Refactored sandbox and path security logic into a shared utility to ensure consistent protection across agent tools and API endpoints.

## [0.9.5] - 2026-05-11

### Added

- **Agent Reflection Service**: Implemented a background service that periodically analyzes session logs to synthesize "Learnings" (facts, preferences, project context) and automatically updates `MEMORIES.md` and searchable memory.
- **Background Logic Loop**: The service runs autonomously in the background across all modes, using the agent's model to act as a "Memory Architect".
- **Database-Driven Discovery**: Integration with `sqlx` to scan `sessions.db` for new activity and ensure each message is processed once.

## [0.9.4] - 2026-05-11

### Added

- **Long-Term Searchable Memory**: Integrated `adk-memory` with a SQLite backend to provide persistent, searchable memory across all agent modes (CLI, Bot, Serve, Browse).
- **Memory Tools**: Implemented `recall_memory` and `add_memory` tools, allowing the agent to autonomously retrieve past context and save new facts.
- **CLI Memory Commands**: Added `/recall` slash command for manual memory search and updated `/memo` to utilize the new database-backed storage.
- **Cross-Session Continuity**: Updated the `AgentRunner` and `Launcher` to support the shared memory service, ensuring consistent knowledge across different interfaces.

## [0.9.3] - 2026-05-10

### Added

- **Workspace Configuration**: Integrated `pnpm-workspace.yaml` in the WebUI directory to explicitly manage `esbuild` dependency, ensuring build consistency across environments.


### Added

- **AI Gateway Integration**: Added support and documentation for using MLflow Deployments as an AI Gateway, enabling load balancing and fallback strategies.
- **Custom Model Base URL**: Updated the agent to respect `base_url` in `config.toml`, allowing integration with custom OpenAI-compatible endpoints like AI Gateways or local LLM servers.
- **Embedded Browse Mode**: Added a new `browse` command that serves the application with a fully integrated WebUI embedded directly into the Rust binary using `rust-embed`.
- **Middleware Path Interception**: Implemented custom Axum middleware in `browse` mode to catch requests for `/` and `/ui` before they reach the router, ensuring the custom UI is served at the root path and bypassing default ADK redirects.
- **Unified Build Automation**: Introduced a root-level `Makefile` to automate the entire build pipeline, including WebUI dependency installation, asset compilation, and Rust binary generation.

### Changed

- **CI/CD Pipeline Enhancement**: Updated the GitHub Actions `daily-build.yml` to leverage the new `Makefile`, ensuring all automated releases include the latest compiled WebUI assets.
- **Deployment Structure Refactor**: Reorganized deployment configurations, moving AI Gateway and Telemetry (OpenTelemetry + MLflow) stacks to a unified `deployment/` directory for better project organization.

## [0.9.1] - 2026-05-10

### Added

- **Specialist Tool Enablement**: Specialist agents (`coder`, `researcher`, `writer`, `ralph`) now have full access to core tools (filesystem, search, wiki, etc.), allowing them to perform autonomous actions like writing files during parallel or loop executions.
- **CLI Input Blocking**: Implemented terminal raw mode during agent thinking and tool calling. This prevents user typing from echoing to the screen, ensuring a clean and focused execution state.
- **Silent ESC Cancellation**: Added support for silent interruption using the `ESC` key. Pressing `ESC` now cancels the current request immediately without displaying any feedback text, while `Ctrl+C` remains for explicit cancellation with a message.
- **System Command Status**: Added "thinking" status indicators and cancellation support to all slash commands (e.g., `/plan`, `/tasks`, `/status`, `/wiki`), providing a consistent experience across all interactions.

### Changed

- **Release Build Optimization**: Configured `Cargo.toml` with high-performance release settings, including Link-Time Optimization (LTO), single codegen-unit, and symbol stripping, significantly reducing binary size and improving runtime efficiency.

## [0.9.0] - 2026-05-09

### Added

- **Threaded Web UI**: Implemented a responsive React + Vite chat interface in `webui/` with persistent thread management, sidebar, and modern aesthetics using Tailwind CSS and Lucide.
- **Backend Integration Plan**: Created `PLAN_WEBUI.md` outlining the integration path for the `adk-server` REST API.
- **Tool Call UI Refinement**: Condensed CLI tool-calling argument display to a single, readable line for improved terminal readability.

### Fixed

- **CLI Interrupts**: Enabled `runner.interrupt` on `ESC`/`Ctrl+C` in CLI mode to ensure background tasks are correctly cancelled.
- **Code Cleanup**: Removed unnecessary parentheses in closure per `cargo check` warnings.
- **Repository Hygiene**: Removed untracked `ref` and `idea` directory/file references from Git index.
- **Documentation**: Polished all book chapters (`book/th/`) to match the "Nami" persona and updated `AGENT.md`.


## [0.8.0] - 2026-05-08

### Added

- **Persistent Task Scheduler**: Implemented a `crontab`-style scheduler that runs in the background. It automatically retries unfinished tasks (using `StateManager` integration) and persists its state in `workspace/scheduler.json`.
- **Schedule Slash Command**: Added the `/schedule` command to register and manage automated tasks with cron expressions.
- **Ralph Wiggum Loop**: Introduced an autonomous agent loop that persists until a goal is achieved.
- **Goal Slash Command**: Added the `/goal` command to trigger the Ralph Wiggum loop with a specific goal and stop condition.
- **Parallel Slash Command**: Introduced the `/parallel` command in CLI mode for easy task delegation and multi-agent orchestration.
- **Enhanced Specialized Agents**: Added `coder`, `researcher`, and `writer` specialists in `src/agent/specialists.rs` to support diverse parallel workloads.
- **Observability Stack**: Integrated OpenTelemetry collector and MLflow for robust tracing and experiment tracking, enabling deep insights into agent behavior.

### Changed

- **Default Model**: Optimized the default model configuration for `gemini-2.5-flash` to ensure high-performance tool calling and responsiveness.
- **Dependency Upgrade**: Upgraded `adk-rust` and all associated `adk-*` crates to version `0.8.0`.
- **Logging**: Enabled `pretty_env_logger` in CLI mode for enhanced developer visibility.
- **Persona & State Management**: Refactored the persona formatting logic in `src/agent/agent.rs` and updated `workspace/AGENT.md` and `workspace/STATE_PROTOCOL.md` for better clarity and efficiency.
- **System Telemetry**: Added OpenTelemetry initialization to `src/main.rs`, supporting external OTLP collectors.

### Fixed

- **Code Cleanup**: Removed redundant `ratatui` and other unused dependencies from `Cargo.toml` and `Cargo.lock` that were no longer required.


## [0.7.0] - 2026-05-07

### Added

- **Enhanced TUI**: Improved CLI experience with better markdown rendering and terminal styling.
- **New Agent Skills**:
    - `Imagen`: Integrated AI image generation capabilities via `.skills/imagen/`.
- **CLI Commands**: Added `/new` slash command in CLI mode to reset the current session.

### Changed

- **Persona & Protocol**: Upgraded the Nami persona and context management protocol for more intelligent interactions.
- **Thai Localization**: Refined Thai translation of technical terms for better consistency.
- **Book Reorganization**: Restructured chapter files and added standard frontmatter for better Obsidian integration.

### Fixed

- **CLI Rendering**: Fixed several UI issues including flickers, excessive newlines, and emoji encoding in various terminal environments.
- **CLI Compilation**: Resolved ownership and type-mismatch compilation errors introduced by new CLI command features.

## [0.6.3] - 2026-05-06

### Changed

- **CLI Command Structure**: Simplified the CLI entry point by removing the optional prompt field and requiring explicit subcommands, improving consistency.
- **Direct Run Mode**: Updated the `run` command to use the standardized `adk-rust` session creation API for more robust and reliable execution.


### Added

- **Structured State Management**: Introduced the `StateManager` tool (`init_task`, `update_task`, `get_task`, `list_active_tasks`) for reliable tracking of long-running processes.
- **State Protocol**: Added `workspace/STATE_PROTOCOL.md` to provide the agent with mandatory guidelines for session continuity.

### Changed

- **Persona Migration**: Moved `AGENT.md`, `USER.md`, and `MEMORIES.md` from the project root to the `workspace/` directory for centralized management.
- **Context Optimization**: Refactored `AGENT.md`, `USER.md`, `MEMORIES.md`, and `STATE_PROTOCOL.md` into high-density, token-efficient formats.
- **Instruction Template**: Optimized the core agent instruction template in `src/agent/agent.rs` to reduce per-turn token overhead.
- **Always-On Protocol**: Integrated the `STATE_PROTOCOL.md` directly into the agent's core system instructions.
- **CLI UX Shorthand**: Enabled direct prompts (e.g., `nami "hi"`) as a shorthand for the `run` command, making the interface more intuitive.
- **Project Initialization**: Updated `init` mode to automatically create token-optimized persona and protocol files within the `workspace/` directory.

### Removed

- **Manual Task Logs**: Deleted the obsolete `TaskLog.md` template and logs in favor of the structured tool-based approach.

## [0.6.1] - 2026-05-05

### Added

- **Interactive Project Initialization**: Refactored the `init` mode to use `inquire`, providing arrow-navigable selection menus for LLM providers and models, and masked input for API keys.
- **Wiki Templates**: Introduced new Markdown templates for `Blog Post` and `Task Log` in `workspace/wiki/Templates/` to standardize recurring wiki entries.

### Changed

- **Code Quality**: Applied `clippy` optimizations across the codebase to ensure idiomatic Rust patterns and better performance.
- **Repository Maintenance**: Removed `mcp.json` from version control to prevent local configuration leakage; updated `mcp.json.example` with remote SSE transport examples.
- **Documentation**: Added future extension ideas to developer tips and improved documentation for the `.namiignore` access control system.

### Fixed

- **Compilation & Stability**: Resolved various compilation issues and type mismatches in the MCP transport layer.
- **OpenAI Schema Validation**: Added missing properties to empty tool argument schemas to prevent OpenAI API validation errors.

## [0.6.0] - 2026-05-05

### Added

- **Pretty CLI Error Rendering**: Implemented a new error display system in CLI mode. Errors are now styled with `crossterm` and include **Intelligent Hints** for common issues like Gemini's `exclusiveMaximum` JSON schema limitations, API quota limits, and authentication failures.
- **Dual MCP Transport Support**: The agent now supports both local `stdio` child processes and remote `streamable HTTP` (SSE) MCP servers. This allows connecting to cloud-hosted MCP services directly via a `url` in `mcp.json`.
- **Obsidian Wiki Automation**: `add_wiki_page` now automatically generates YAML frontmatter (title, date, tags) and ensures a level-1 Markdown header is present if missing. It also enforces Title Case with spaces for filenames.
- **`.namiignore` Access Control**: Implemented a glob-based ignore system for filesystem tools. The agent now respects `.namiignore` patterns (defaulting to `.git/`, `target/`, `.env`, and `sessions.db`) to prevent unauthorized file access within the workspace.

### Changed

- **MCP Tool Namespacing**: All tools imported from MCP servers are now prefixed with `mcp_` (e.g., `mcp_read_file`) using `PrefixedToolset` to resolve naming collisions with built-in system tools.
- **Persona Optimization**: Updated `AGENT.md` and `MEMORIES.md` to enforce strict plain text output for chat responses (optimized for Telegram) while maintaining standard Markdown for the wiki vault and file editing.
- **Project Synchronization**: Synchronized `AGENT.md`, `USER.md`, and `MEMORIES.md` to align with the new plain-text and Obsidian-formatting rules.

### Fixed

- **Tool Name Collisions**: Resolved a critical "Duplicate tool name 'read_file'" error when loading MCP servers.

## [0.5.0] - 2026-05-04

### Added

- **Ebook Creation Skill**: Introduced the `create-ebook` skill, enabling users to generate PDF and EPUB files directly from a directory of Markdown files. This includes automatic page breaks between chapters.

### Changed

- **Skill Prioritization**: Updated agent instructions to prioritize specialized skills over general tools, improving the accuracy and efficiency of task execution.
- **Tool Renaming**: Renamed `system_info` tool and skill to `system_status` for better clarity.

### Fixed

- **Compilation Errors**: Resolved compilation errors in `agent.rs` and verified the build.

## [0.4.0] - 2026-05-03

### Added

- **@ File Context References**: Implemented an interactive file referencing system in the CLI. Users can type `@` followed by a file path to inject file contents directly into the prompt. Includes built-in tab-completion powered by `rustyline` scoped to the `workspace/` sandbox.
- **Parallel Task Tool**: Added a custom `parallel_tasks` orchestrator that enables the agent to trigger multiple sub-agents simultaneously for faster multi-task processing.
- **Wiki Management Tools**: Added `get_backlinks`, `apply_template`, `check_broken_links`, and `rename_wiki_page` to enhance Obsidian-style knowledge management.
- **Daily Notes Template**: Added a default `DailyTemplate.md` for consistent daily journaling.

### Changed

- **Project Structure**: Refactored the codebase directory structure. Moved entry points into a dedicated `src/modes/` directory and relocated tools and utilities to `src/tools/` and `src/utils/` for better separation of concerns.
- **Wiki Search**: Upgraded `search_wiki` and `search_wiki_by_tag` to support regex and YAML frontmatter parsing.
- **CLI & Docs**: Changed CLI version greeting to be dynamically retrieved from `Cargo.toml` (v0.4.0).

### Fixed

- **Compilation Errors**: Resolved module pathing and type inference issues related to `get_workspace_dir` and the `rustyline` upgrade following the directory restructure.

### Removed

- **Wiki Date Search**: Removed `search_wiki_by_date` in favor of more robust tag and content search strategies.

## [0.3.0] - 2026-05-03

### Added

- **Hierarchical Sub-Agents**: Introduced an ecosystem of 7 specialized agents (Codebase Investigator, Generalist, Web Developer, DevOps Engineer, Quality Assurance, Data Specialist, Documentation Architect) to support complex task delegation.
- **Obsidian-Style Wiki**: Implemented new wiki tools including `get_wiki_graph` for knowledge graph visualization, `search_wiki_by_tag` for tag-based search, and `create_daily_note` for daily journaling, enhancing bi-directional linking capabilities.

## [0.2.0] - 2026-05-02

### Added

- **`init` command**: Automates project initialization by creating essential configuration files (`AGENT.md`, `MEMORIES.md`, `USER.md`) and bootstrapping the `sessions.db` database schema.
- **Event Compaction**: Implemented automatic conversation history compaction to manage memory and performance.
- **Configurable Compaction**: Added support for configuring compaction settings in serve mode.

### Refactored

- **Agent Configuration**: Centralized compaction logic within the agent module.
- **AgentRunner**: Improved runner architecture to support event compaction and better task management.

### Fixed

- **CLI/Runner**: Improved robustness of CLI and tool execution; added support for ESC cancellation in interactive mode.
- **Help Text**: Enhanced CLI usability with improved help formatting.

## [0.1.0] - Initial Release

- Initial project setup as a Telegram AI Bot (namiClaw) with support for persistent sessions, Wiki KM, and modular tool architecture.
