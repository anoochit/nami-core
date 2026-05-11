# Agent Module

## Purpose
The `agent` module orchestrates the core AI agent logic, managing configuration, persona, tool registration, and the lifecycle of the agent builder. It serves as the primary gateway for defining Nami's personality and capabilities.

## Architecture & Responsibilities
- **`agent.rs`**: Core factory for agent construction, persona formatting, and instruction management.
- **`specialists.rs`**: Logic for managing and injecting specialist agent tools (e.g., coder, writer).
- **`mcp.rs`**: Handles loading and registration of Model Context Protocol (MCP) servers and their tools.
- **`reflection.rs`**: Background service that synthesizes conversational history into persistent insights.

## Key Entry Points
- `create_agent(&AppConfig)`: Main factory function for building a configured agent.
- `build_agent()`: Wrapper to load `config.toml` and initialize the agent setup process.
- `format_persona(...)`: Assembles the system prompt template.

## Dependencies
- **External**: `adk-rust`, `adk-runner`
- **Internal**: `crate::tools`, `crate::utils`

## Maintenance Note
- When adding new specialized agents, ensure they are registered in `specialists.rs` and added to the `core_tools` list in `agent.rs`.
- System instructions (`format_persona`) must remain concise to stay within token limits.
