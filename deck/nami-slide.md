---
marp: true
theme: default
paginate: true
header: 'nami: AI Bot'
footer: 'Powered by namiClaw'
---

# nami: AI Bot
### A Modular, Extensible AI Agent Framework

Building sophisticated AI agents with persistent sessions, filesystem sandbox, and dynamic persona management.

Built on **adk-rust** and **teloxide**.

---

# 🚀 What is Nami?

- **Multi-Platform AI**: Support for Gemini, Anthropic, or OpenAI-compatible LLMs.
- **Persistent Sessions**: SQLite-backed history.
- **Hybrid MCP Support**: Connect to local and remote Model Context Protocol servers.
- **Sandboxed Environment**: Filesystem tools protected by `.namiignore`.

---

# ✨ Core Features

- **Modern TUI**: Rich interactive CLI with ASCII banners and animated indicators.
- **@ File Context**: Reference workspace files directly using `@path/to/file`.
- **Parallel Task Execution**: Orchestrate multiple sub-agents simultaneously.
- **Live Web Search**: Integrated Google Search via Serper.dev.

---

# 🏗 Architecture

The system supports multiple entry points sharing core agent logic:

- **Telegram Bot**: via teloxide.
- **Interactive CLI**: Rich terminal experience.
- **Direct Run**: Execute single prompts.
- **HTTP Server**: Run as a service.

*Core logic powered by adk-rust and LlmAgent.*

---

# 📚 Obsidian-Style Wiki KM

Transparent Knowledge Management using Markdown files:

- **Graph View**: Knowledge graph nodes and edges.
- **Tagging**: Filter notes by `#tags`.
- **Daily Notes**: Automatic journal entries.
- **Backlinks & Links**: Support for `[[wikilink]]` and link integrity checks.
- **Templates**: Structured page application.

---

# 👤 Persona & Memories

- **AGENT.md**: Defines the "Soul" (personality and tone).
- **USER.md**: Defines the master's context and preferences.
- **MEMORIES.md**: Automatically records personal facts learned about the user.
- **Todo Management**: Integrated task tracking (`add_todo`, `list_todos`).

---

# ⚙️ Getting Started

1. **Configure**: Set up `.env` with API keys (Google, Teloxide, etc.).
2. **Customize**: Edit `AGENT.md` and `USER.md`.
3. **Build**: `cargo build --release`.
4. **Run**:
   - `nami init`
   - `nami bot`
   - `nami cli`

---

# 💡 Future Vision

- **RAG Integration**: Vector database connection for semantic memory.
- **Multi-modal**: Vision tools to analyze images and screenshots.
- **Voice Mode**: Whisper integration for voice-to-text.
- **Automated Evals**: Accuracy testing suite.
