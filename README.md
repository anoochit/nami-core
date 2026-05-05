# nami: AI Bot

A modular, extensible AI-powered `Nami` built on top of [adk-rust](https://github.com/zavora-ai/adk-rust) and the [teloxide](https://github.com/teloxide/teloxide) framework. This project demonstrates how to leverage modern Rust libraries to build sophisticated AI agents with persistent sessions, filesystem sandbox capabilities, and dynamic persona management.

![Screenshot](screenshots/nami-avatar.png)

## 🚀 Features

* **Multi-Platform AI**: Powered by Gemini, Anthropic, or any OpenAI-compatible LLM (e.g., ThaiLLM).
* **Modern TUI**: A rich, interactive CLI experience with a custom ASCII banner, animated indicators, pretty error rendering with intelligent hints, and structured layout.
* **@ File Context References**: Reference files from the `workspace/` directly in the CLI using `@path/to/file` with built-in Tab-completion.
* **MCP Integration**: Seamlessly connect to [Model Context Protocol](https://modelcontextprotocol.io/) servers. Tools are automatically namespaced with `mcp_` to prevent collisions.
* **Parallel Task Execution**: A custom `parallel_tasks` tool that orchestrates multiple sub-agents simultaneously for high-speed multi-tasking.
* **Markdown Wiki KM**: A transparent, human-readable Knowledge Management system using `.md` files, featuring automatic Obsidian-style frontmatter and header generation.
* **Dynamic Persona & Soul**: Configure the bot's personality and user context via `AGENT.md` and `USER.md`.
* **Persistent Sessions**: SQLite-backed conversation history keyed by Telegram user ID.
* **Modular Tools**: Organized architecture for adding capabilities (Weather, Search, Shell, Wiki, etc.).
* **Live Web Search**: Integrated Google Search via Serper.dev.
* **Todo Management**: Integrated task tracking and list management.
* **Sandboxed Environment**: Integrated filesystem tools for agent tasks within a `workspace/` directory, including the ability to merge multiple documents together using `merge_files`.

## 🛠 Prerequisites

* Rust ([rustup](https://rustup.rs/))
* A Telegram Bot Token from [@BotFather](https://t.me/BotFather)
* API Key for your chosen LLM (Gemini, OpenAI, or ThaiLLM)
* (Optional) [Serper.dev](https://serper.dev/) API Key for Google Search features.

## ⚙️ Configuration

1. Copy `.env.example` to `.env` and configure your credentials:

```bash
cp .env.example .env
```

```text
GOOGLE_API_KEY=your_google_api_key_here
THAILLM_API_KEY=your_api_key_here
TELOXIDE_TOKEN=your_telegram_bot_token
SERPER_API_KEY=your_serper_api_key
```

1. Customize the Bot's Soul:

* Edit `AGENT.md` to change the name, personality, and tone.
* Edit `USER.md` to provide context about yourself and your preferences.

## 🏃 Getting Started

### Build and Install

1. **Build the application**:

   ```bash
   cargo build --release
   ```

   The generated executable will be found in `target/release/`.

2. **(Optional) Install globally**:
   To run `nami` from any directory, you can move the binary to a location in your system's `PATH`:

   * **Linux/macOS**:

     ```bash
     sudo mv target/release/nami /usr/local/bin/
     ```

   * **Windows**:
     Add the full path of the `target\release\` directory to your system's Environment Variables (PATH).

### Running

The application provides five primary run modes:

| Mode | Command | Description |
| :--- | :--- | :--- |
| **Initialize** | `nami init` | Initialize project config files and database. |
| **Telegram Bot** | `nami bot` | Start the interactive Telegram Bot. |
| **CLI** | `nami cli` | Local interactive terminal agent with rich TUI. |
| **Run** | `nami run "prompt"` | Execute a single prompt directly from the CLI. |
| **Server** | `nami serve` | Run as an HTTP service. |

## 🏗 Architecture

The system supports multiple entry points sharing the same core agent logic:

```mermaid
graph TD
    subgraph EntryPoints [Modes]
        direction TB
        Bot[Telegram Bot]
        CLI[Interactive CLI]
        Run[Direct Run]
        Server[HTTP Server]
    end

    EntryPoints --> Runner[adk-rust Runner]
    
    Runner --> Agent[LlmAgent]
    Runner --> DB[(SqliteSessionService)]
    
    Agent --> LLM[ThaiLLM/Gemini/OpenAI]
    Agent --> SubAgents[Sub-Agents: Generalist]
    Agent --> Tools[Tools]
    Agent --> Wiki[Obsidian-Style Wiki: Graph, Tags, Daily Notes]
    Agent --> Persona[AGENT.md & USER.md]
    
    SubAgents --> Agent
```

* **teloxide**: Handles Telegram polling and updates.
* **adk-rust**: Core framework for AI agent logic and memory management.
* **SqliteSessionService**: Provides persistent session storage (`sessions.db`).
* **Modes**: Located in `src/modes/`, contains application entry points (Bot, CLI, Server, etc.).
* **Tools**: Located in `src/tools/`, contains all functional modules.

## 🧩 Extensions

### Obsidian-Style Wiki Knowledge Management

The bot uses the `wiki/` directory in its workspace to store long-term knowledge, now with Obsidian-style features:

* `add_wiki_page`: Saves new information as Markdown, with support for `[[wikilink]]` syntax.
* `get_wiki_graph`: Generates a JSON representation of your knowledge graph's nodes and edges.
* `search_wiki_by_tag`: Filters notes by specific `#tags`.
* `create_daily_note`: Creates a journal entry for the current date.
* `summarize_wiki`: Generates a `SUMMARY.md` index of all topics.
* `search_wiki`: Full-text search across all knowledge pages.
* `get_backlinks`: Lists all pages that link to a specific note.
* `apply_template`: Applies a structured template to a wiki page.
* `check_broken_links`: Identifies and reports dead wikilinks.
* `rename_wiki_page`: Safely renames a page and updates all incoming links.

### Todo Management

The bot features a built-in task manager for tracking goals and daily items.

* `add_todo`: Create new tasks.
* `list_todos`: View current pending items.
* `mark_todo_done`: Mark tasks as finished.
* `remove_todo`: Permanently delete a task.

### Persona & Memories

* **AGENT.md**: Defines the "Soul" of the bot.
* **USER.md**: Defines the context of the master.
* **MEMORIES.md**: Automatically updated by the bot when it learns personal facts about the user.

### Publishing Skills

The bot includes built-in skills to compile your workspace documents into distributable formats:

* `create-pdf`: Converts Markdown files into beautifully formatted PDF documents (requires `md-to-pdf`).
* `create-epub`: Compiles Markdown files into EPUB e-books with automatic BOM sanitization (requires `md-to-epub`).

## 💡 Developer Tips

* **Production**: For high-traffic bots, migrate `teloxide` from polling to webhooks.
* **Future Extension Ideas**:
    * **RAG Integration**: Connect the `wiki/` vault to a vector database (like Qdrant or Milvus) for semantic search and long-term memory retrieval.
    * **Vision & Multi-modal**: Enable vision tools to allow Nami to analyze screenshots or images sent via Telegram.
    * **Voice Mode**: Integrate Whisper for voice-to-text, allowing you to talk to Nami directly.
    * **Automated Evaluations**: Build an `eval/` suite to test Nami's tool-calling accuracy across different model versions.
