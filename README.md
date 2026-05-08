# nami: AI Bot

A modular, extensible AI-powered `Nami` built on top of [adk-rust](https://github.com/zavora-ai/adk-rust) and the [teloxide](https://github.com/teloxide/teloxide) framework. This project demonstrates how to leverage modern Rust libraries to build sophisticated AI agents with persistent sessions, filesystem sandbox capabilities, and dynamic persona management.

![Screenshot](screenshots/nami-avatar.png)

## 🚀 Features & Capabilities

### 🧠 Core Intelligence & Orchestration
*   **Multi-Platform AI**: Powered by Gemini, Anthropic, or any OpenAI-compatible LLM (e.g., ThaiLLM).
*   **Parallel Task Execution**: A custom `parallel_tasks` tool that orchestrates multiple sub-agents (`coder`, `researcher`, `writer`, `generalist`, `ralph`) simultaneously for high-speed multi-tasking.
*   **Autonomous Goal Loops**: A "Ralph Wiggum" loop agent that persists through multiple iterations to achieve complex goals, triggered via `/goal`.
*   **Hybrid MCP Support**: Seamlessly connect to both local (stdio-based) and remote (streamable HTTP/SSE) [Model Context Protocol](https://modelcontextprotocol.io/) servers. Tools are automatically namespaced with `mcp_` to prevent collisions.
*   **Hierarchical Sub-Agents**: Ecosystem of specialized agents (Codebase Investigator, Generalist, Web Developer, DevOps Engineer, Quality Assurance, Data Specialist, Documentation Architect) for complex task delegation.

### 💻 Rich User Interface
*   **Modern TUI**: A rich, interactive CLI experience with a custom ASCII banner, animated indicators, pretty error rendering with intelligent hints, and structured layout.
*   **Slash Commands**: Quick access to system functions:
    *   `/new`: Reset current session.
    *   `/parallel`: Run tasks in parallel.
    *   `/goal`: Run autonomous loops.
    *   `/plan`: Initialize structured tasks.
*   **@ File Context References**: Reference files from the `workspace/` directly in the CLI using `@path/to/file` with built-in Tab-completion.
*   **Dynamic Persona & Soul**: Configure the bot's personality and user context via `workspace/AGENT.md` and `workspace/USER.md`. Automatically updated `workspace/MEMORIES.md` tracks personal user facts.

### 📂 Knowledge & Session Management
*   **Obsidian-Style Wiki KM**: A transparent, human-readable Knowledge Management system using `.md` files.
    *   `add_wiki_page`: Markdown saving with `[[wikilink]]` support.
    *   `get_wiki_graph`: Knowledge graph visualization.
    *   `search_wiki_by_tag`: Filter notes by specific `#tags`.
    *   `create_daily_note`: Journal entries for the current date.
    *   `get_backlinks`: List pages linking to a specific note.
    *   `rename_wiki_page`: Safe renaming with link updates.
*   **Persistent Sessions**: SQLite-backed conversation history keyed by Telegram user ID.
*   **State Management**: A structured JSON-based system for tracking long-running tasks, guided by `workspace/STATE_PROTOCOL.md`.
    *   `init_task`: Initialize new processes with goals and steps.
    *   `update_task`: Progress tracking and persistent context.
    *   `list_active_tasks`: View all in-progress or blocked tasks.
*   **Todo Management**: Built-in task manager for tracking goals and daily items (`add_todo`, `list_todos`, `mark_todo_done`).

### 🛠 Specialized Skills & Tools
*   **AI Image Generation**: Integrated `Imagen` capabilities via `.skills/imagen/` for high-quality image creation.
*   **Expert Code Analysis**: `Vercel React Best Practices` suites for deep performance and security analysis of React applications.
*   **Publishing Skills**: Compile workspace documents into distributable formats:
    *   `create-pdf`: Beautifully formatted PDF documents.
    *   `create-epub`: EPUB e-books with BOM sanitization.

### 🛡 System & Safety
*   **Sandboxed Environment**: Integrated filesystem tools for agent tasks within a `workspace/` directory, protected by a **`.namiignore` policy** (similar to `.gitignore`) to control access permissions.
*   **Observability Stack**: Integrated OpenTelemetry collector and MLflow for robust tracing and experiment tracking.
*   **Live Web Search**: Integrated Google Search via Serper.dev.
*   **Modular Architecture**: Organized structure for adding capabilities (Weather, Search, Shell, Wiki, etc.).

## 💡 Developer Tips

* **Production**: For high-traffic bots, migrate `teloxide` from polling to webhooks.
* **Future Extension Ideas**:
    * **RAG Integration**: Connect the `wiki/` vault to a vector database (like Qdrant or Milvus) for semantic search and long-term memory retrieval.
    * **Vision & Multi-modal**: Enable vision tools to allow Nami to analyze screenshots or images sent via Telegram.
    * **Voice Mode**: Integrate Whisper for voice-to-text, allowing you to talk to Nami directly.
    * **Automated Evaluations**: Build an `eval/` suite to test Nami's tool-calling accuracy across different model versions.
