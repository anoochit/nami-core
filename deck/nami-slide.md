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

Built on **adk-rust**, **teloxide**, and **axum**.

---

# 🚀 What is Nami?

- **Omni-Channel AI**: Support for Telegram, LINE, CLI, and Web UI.
- **Multi-Model Support**: Gemini, Anthropic, or OpenAI-compatible (ThaiLLM).
- **Persistent Memory**: SQLite-backed history and long-term searchable facts.
- **Hybrid MCP Support**: Connect to local and remote Model Context Protocol servers.
- **Sandboxed Environment**: Filesystem tools protected by `.namiignore`.

---

# ✨ User Experience

- **Modern TUI**: Rich interactive CLI with ASCII banners and animated indicators.
- **Embedded Web UI**: React-based dashboard via `nami browse`.
- **Slash Commands**: Quick actions like `/new`, `/parallel`, `/goal`, and `/schedule`.
- **@ File Context**: Reference workspace files directly using `@path/to/file`.
- **Silent Cancellation**: Interruption support with `Ctrl+C` or `ESC`.

---

# 🏗 Agent Intelligence

- **Sub-Agent Specialists**: Dedicated agents for Coding, Research, Writing, and more.
- **Parallel Orchestration**: Execute and monitor multiple complex tasks simultaneously.
- **Autonomous Loops**: "Ralph Wiggum" loop agent for multi-step goal attainment.
- **Agent Reflection**: Background service synthesizes session logs into persistent memories.
- **AI Gateway**: High-availability routing and fallback via MLflow Deployments.

---

# 📚 Knowledge & State

- **Obsidian-Style Wiki**: Human-readable Markdown vault with `[[wikilinks]]`.
  - **Graph View**: Knowledge graph nodes and edges.
  - **Tagging & Backlinks**: Advanced note discovery and integrity.
  - **Daily Notes**: Automatic journaling for project tracking.
- **State Protocol**: Structured JSON tracking for long-running task persistence.
- **Todo Manager**: Integrated task tracking (`add_todo`, `list_todos`).

---

# 👤 Soul & Skills

- **Dynamic Persona**: Personality and context defined in `AGENT.md` and `USER.md`.
- **Modular Skills**: Extensibility via `workspace/.skills/` (e.g., Blog Manager, System Status).
- **Publishing Suite**: Automate documentation into **PDF** or **EPUB** formats.
- **Native Image Gen**: High-quality visuals via Gemini 2.5 Flash Image.

---

# 🛠 System & Observability

- **Observability Stack**: Integrated **OpenTelemetry** and **MLflow** for tracing and tracking.
- **Task Scheduler**: Persistent `crontab`-style background system for automated tasks.
- **Performance Optimized**: Tuned release profiles with LTO and symbol stripping.
- **Live Web Search**: Integrated Google Search via Serper.dev.

---

# ⚙️ Getting Started

1. **Configure**: Set up `.env` with API keys (Google, Teloxide, LINE, etc.).
2. **Customize**: Edit `AGENT.md` and `USER.md`.
3. **Build**: `make build` (compiles WebUI assets and Rust binary).
4. **Run**:
   - `nami init`
   - `nami bot` / `nami line`
   - `nami cli` / `nami browse`

---

# 💡 Future Vision

- **RAG Integration**: Vector database connection for advanced semantic memory.
- **Voice Mode**: Whisper integration for voice-to-text and speech synthesis.
- **Multi-modal Tools**: Deep vision analysis for images and technical screenshots.
- **Automated Evals**: Continuous accuracy testing and scoring suite.
