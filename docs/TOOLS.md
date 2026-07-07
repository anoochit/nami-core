# Tool System & MCP Integration

Nami's capabilities are extended through a modular tool system, including built-in tools and external integrations via the Model Context Protocol (MCP).

## 🛠 Tool Architecture

Tools in Nami implement the `Tool` trait from the `adk-rust` crate. Each tool defines:
-   **Name & Description**: Used by the LLM for discovery.
-   **Parameter Schema**: A JSON Schema defining the expected input.
-   **Execution Logic**: The asynchronous Rust code that performs the action.

## 🛡 Security & Sandboxing (`src/utils/mod.rs`)

To protect the host system, Nami enforces a strict sandboxing policy for all filesystem-related tools.

-   **Workspace Isolation**: All file operations are restricted to the `workspace/` directory.
-   **Path Neutralization**: The `sandbox()` utility strips leading slashes and prevents directory traversal (e.g., `../../`).
-   **.namiignore**: A policy file (similar to `.gitignore`) that explicitly denies access to sensitive files or directories within the workspace.
-   **Unit Tests**: The sandboxing logic is verified by automated tests in `src/utils/mod.rs`.

## 🔌 Model Context Protocol (MCP) (`src/agent/mcp.rs`)

Nami acts as an MCP client, allowing it to connect to external tool servers.

-   **Transports**:
    -   **Stdio**: Spawns local processes and communicates via standard I/O.
    -   **HTTP**: Connects to remote MCP servers over SSE (Server-Sent Events).
-   **Configuration**: MCP servers are defined in `mcp.json`.
-   **Schema Sanitization**: The `SanitizedTool` wrapper automatically removes Gemini-incompatible fields (like `x-` extensions) from MCP tool schemas to ensure provider compatibility.
-   **Prefixing**: MCP tools are automatically prefixed with `mcp_` to avoid naming collisions with built-in tools.

## 📦 Built-in Tools

-   **Filesystem**: `read_file`, `write_file`, `list_files`, `search_files`.
-   **Web**: `web_fetch` (summarizes web content), `wiki` (manages knowledge base).
-   **Memory**: `recall_memory`, `add_memory`.
-   **System**: `execute_shell` (restricted).
-   **Soul**: `update_user_memory` (updates agent's view of the user), `update_agent_soul` (evolves agent instructions/personality).
