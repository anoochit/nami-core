# Tool System & MCP Integration

Nami's capabilities are extended through a modular tool system, including built-in tools and external integrations via the Model Context Protocol (MCP).

## 🛠 Tool Architecture

Tools in Nami implement the `Tool` trait from the `adk-rust` crate. Each tool defines:

- **Name & Description**: Used by the LLM for discovery.
- **Parameter Schema**: A JSON Schema defining the expected input.
- **Execution Logic**: The asynchronous Rust code that performs the action.

## 🛡 Security & Sandboxing (`src/utils/paths.rs`, `src/utils/ignore.rs`)

To protect the host system, Nami enforces a strict sandboxing policy for all filesystem-related tools and commands.

- **Workspace Isolation**: All file operations are restricted to the `workspace/` directory (or custom `NAMI_WORKSPACE`).
- **Path Neutralization**: The `sandbox()` utility in `src/utils/paths.rs` strips leading slashes and prevents directory traversal (e.g., `../../`).
- **.namiignore**: A policy file (similar to `.gitignore`) handled in `src/utils/ignore.rs` that explicitly denies access to sensitive files or directories within the workspace.
- **Shell Execution Controls**: The shell execution tool (`execute_shell`) can be customized via `[tools.shell]` configurations:
  - **`allowed_commands`**: Whitelist of shell executables permitted to run.
  - **`blocked_commands`**: Specific commands explicitly blocked from running.
  - **`security_level`**: Enforces `strict` (runs only explicitly whitelisted commands) or `permissive` (runs anything not explicitly blocked) boundaries.
- **Unit Tests**: The sandboxing and shell filtering logic is verified by automated tests.

## 📦 Modular Loading & Centralized Factory (`src/tools/mod.rs`)

Rather than hard-coding tools, Nami registers core tools through a modular, centralized `create_core_tools` factory. It utilizes a `ToolFactoryConfig` (defined in `config.toml`) to conditionally load tools in logical categories (e.g., `filesystem`, `web`, `media`, `generation`, `system`), making Nami highly customizable and lightweight.

## 🔌 Model Context Protocol (MCP) (`src/agent/mcp.rs`)

Nami acts as an MCP client, allowing it to connect to external tool servers.

- **Transports**:
  - **Stdio**: Spawns local processes and communicates via standard I/O.
  - **HTTP**: Connects to remote MCP servers over SSE (Server-Sent Events).
- **Configuration**: MCP servers are defined in `mcp.json`.
- **Schema Sanitization**: The `SanitizedTool` wrapper automatically removes Gemini-incompatible fields (like `x-` extensions) from MCP tool schemas to ensure provider compatibility.
- **Prefixing**: MCP tools are automatically prefixed with `mcp_` to avoid naming collisions with built-in tools.

## 📦 Built-in Tools

- **Filesystem**: `read_file`, `write_file`, `list_files`, `search_files`, `delete_file`, `analyze_media` (natively processes images, audio, video, and PDFs).
- **Web**: `web_fetch` (automatically parses raw HTML to markdown format utilizing `html2md`), `wiki` (manages knowledge base).
- **Memory**: `recall_memory`, `add_memory`.
- **Generators**: `image_generator`, `audio_generator`, `video_generator`.
- **System**: `execute_shell` (bound by security configurations), `current_datetime`, `scheduler`, `todo`.
- **Delegation**: `supervised_delegate` (DAG task orchestration), `invoke_agent` (single specialist delegation), `parallel_tasks` (parallel multi-specialist execution).
- **Soul & Evolution**: `update_user_memory` (updates agent's view of the user), `update_agent_soul` (evolves agent instructions/personality), `evolution`.
- **Artifacts**: `load_artifacts` (loads versioned binary files).
