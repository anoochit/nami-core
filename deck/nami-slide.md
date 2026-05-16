---
marp: true
theme: default
paginate: true
header: 'nami: AI Framework'
footer: 'Next-Gen Agentic Intelligence'
---

# nami
### The Modular Framework for Enterprise AI Agents

Building sophisticated, agentic workflows with **persistence**, **sandbox security**, and **dynamic personality**.

Built on **adk-rust**, **teloxide**, and **axum**.

---

# 🚀 Meet Nami

- **Omni-Channel**: Telegram, LINE, CLI, and Web UI.
- **Model Agnostic**: Gemini, Anthropic, or OpenAI (ThaiLLM).
- **Persistent Memory**: SQLite-backed history and context.
- **Hybrid MCP**: Native local/remote tool orchestration.
- **Secure by Design**: Filesystem sandboxing with `.namiignore`.

---

# ✨ User-Centric Design

- **Modern TUI**: Interactive CLI with structured feedback.
- **Embedded Web UI**: Instant dashboard via `nami browse`.
- **Velocity Commands**: Fast actions like `/goal`, `/parallel`, `/schedule`.
- **Intuitive Context**: `@` file references with Tab-completion.
- **Seamless Flow**: Silent `ESC` cancellation and raw mode input.

---

# ⚡ v0.9.15: Latest Advancements

- **WebUI Modularity**: Refactored `ThreadView.tsx` for scale.
- **Artifact Preview**: Instant file visualization for agents.
- **Tooling Enhancements**: Integrated preview-ready artifact headers.
- **Reliability Sync**: Continuous remote integration.

---

# 🏗 Agentic Intelligence

- **Specialist Ecosystem**: Dedicated agents for Coding, Research, and Writing.
- **Parallel Orchestration**: Multi-tasking at scale.
- **Autonomous Loops**: Goal-driven agents ("Ralph Wiggum") for complex problem solving.
- **Memory Architect**: Background reflection for synthesis and context-building.
- **AI Gateway**: Enterprise-grade load balancing via MLflow.

---

# 🧠 Knowledge & Memory

- **Transparent Wiki**: Obsidian-compatible Markdown vault.
  - Knowledge graph visualization.
  - Tagging, Backlinks, and integrity checks.
- **Persistent State**: JSON-based protocols for long-running workflows.
- **Task Management**: Integrated Todo & Scheduler suite.

---

# 👤 The Nami Persona

- **Configurable Soul**: Persona and Context via `AGENT.md` and `USER.md`.
- **Modular Skills**: Extensible via `workspace/.skills/`.
- **Native Generation**: High-quality visuals using Gemini 2.5 Flash Image.
- **Publishing Suite**: Automate PDF/EPUB distribution.

---

# 🛠 Engineering & Observability

- **Observability Stack**: Full tracing with **OpenTelemetry** and **MLflow**.
- **Robust Scheduling**: Background automated tasks.
- **Performance Optimized**: LTO, symbol stripping, and low-footprint runtime.
- **Integrated Intelligence**: Real-time Google Search capability.

---

# 🔮 The Vision

- **Semantic Memory**: RAG integration for deep knowledge retrieval.
- **Voice-First**: Whisper integration for natural speech interaction.
- **Vision-Agentic**: Deep analysis for images and technical UI contexts.
- **Continuous Evals**: Automated accuracy benchmarking.

---

# 🚀 Get Started

1. **Configure**: Set up `.env` (Google, Teloxide, LINE).
2. **Personalize**: Edit `AGENT.md` & `USER.md`.
3. **Build**: `make build` (compiles all assets).
4. **Deploy**:
   - `nami init`
   - `nami bot` or `nami browse`

