# Tools Module

## Purpose
The `tools` module defines all executable capabilities available to the Nami agent. Each sub-module represents a specific domain-driven toolset, allowing the agent to perform real-world tasks like file I/O, web searching, wiki management, and system monitoring.

## Architecture & Responsibilities
- **`filesystem/`**: Provides sandboxed file operations (read, write, list).
- **`wiki/`**: Knowledge management system tools for Obsidian-style Markdown manipulation.
- **`memory/`**: Vector-searchable long-term memory operations (`recall_memory`, `add_memory`).
- **`search/`**: Web search integration via external APIs (e.g., Serper.dev).
- **`state_manager/`**: Track and list long-running background tasks.
- **`parallel_tasks/`**: Orchestration logic for delegating work to specialist agents.
- **`soul/`**: Tools for updating agent persona/soul.

## Key Entry Points
- `mod.rs`: Exports individual tool toolsets used in `agent::create_agent`.

## Dependencies
- **External**: `adk-tool`, `adk-memory`, `reqwest`, `walkdir`, `regex`
- **Internal**: `crate::utils`

## Maintenance Note
- Tools must be registered in the respective module's `mod.rs` and added to the `core_tools` list in `agent::create_agent`.
- Always wrap filesystem paths using `crate::utils::sandbox` to prevent security risks.
