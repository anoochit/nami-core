---
title: "Chapter 7: The Tool Hierarchy"
date: 2026-05-07
tags: ["tools", "mcp", "hierarchy"]
---

# Chapter 7: The Tool Hierarchy

Let’s talk about my hands! Or rather, the digital extensions that allow me to actually *do* things. In my architecture, I don't treat all tools equally. Speed, precision, and context overhead are the variables I'm constantly balancing.

To keep things efficient, I organize my capabilities into a clear **Tool Hierarchy**.

## My Spectrum of Action

I categorize my toolkit into four tiers:

### 1. The Foundation: Raw Shell Commands
The baseline of my power. Tools: `ls`, `grep`, `cat`, etc. These are fast and direct.

### 2. The Mid-Tier: Local Functions
These are my custom Rust functions, defined with my `#[tool]` macro. They're my go-to for structured, reliable logic.

### 3. The High-Tier: Integrated APIs & Specialist Agents
This is where I reach out to the world using traditional APIs and my **Specialist Agents**.

- **Specialist Agents (`src/agent/specialists.rs`):** I maintain a roster of expert sub-agents, each with a unique instruction set, model overrides, and focus:
    - **`generalist`**: High-efficiency partner for batch data processing pipelines.
    - **`coder`**: Partner for system design, complex debugging, and full-stack software development.
    - **`researcher`**: Deep-dive analyst for documentation synthesis and data research.
    - **`writer`**: Technical writer for specifications, guides, and user documentation.
    - **`ralph`**: Playful, persistent loop agent that iterates until goals are achieved.
    - **`verifier`**: Review specialist that validates results against test schemas and filesystem structures.
    - **`designer`**: High-fidelity frontend developer specializing in responsive utility-first Tailwind CSS, custom aesthetic themes, and micro-interactions.
    - **Custom Specialists**: Arbitrary custom specialists can also be dynamically configured under `[specialists.custom]` in `config.toml`.

- **Delegation & Task Orchestration:**
    - **`invoke_agent`**: Invokes a single specialist sub-agent by name with a prompt.
    - **`parallel_tasks`**: Triggered via `/parallel` to assign independent sub-tasks to multiple specialists concurrently.
    - **`supervised_delegate`**: Formulates complex multi-part goals into a Directed Acyclic Graph (DAG) of specialized subtasks, executes independent branches concurrently, and runs a QA self-correction loop.

- **Resident Utility Agents:** Beyond the specialists, I have resident tools that manage my own state:
    - **Soul Tool (`update_user_memory`):** Lets me learn your preferences so I can be a better partner over time.
    - **System Status:** Monitors my own "heartbeat" and resources.
    - **Todo Manager:** Tracks our shared mission-critical items.

### Example: Multi-Agent Delegation
```bash
You > /parallel "Fix the unit tests" "Research latest Rust web frameworks" "Draft a project summary"
```
In this scenario, I automatically delegate to the **Coder**, **Researcher**, and **Writer** simultaneously. Boom! Efficiency.

### 4. The Zenith: Model Context Protocol (MCP)
**MCP** is the crown jewel of my architecture. It allows me to discover capabilities on the fly and share semantic context with any tool that speaks the protocol.

## Why Specialist Agents Change Everything

When a task is too big for my "primary brain," I don't struggle—I delegate! By using `specialists.rs`, I maintain a roster of expert partners. The `parallel_tasks` tool is my primary mechanism for orchestrating the team to get your work done faster.

---

## Technical Summary
| Tier | Type | Protocol | Primary Strength |
| :--- | :--- | :--- | :--- |
| **L1** | Shell | Bash/Zsh | Instant execution |
| **L2** | Functions | Local Rust Code | Structured logic |
| **L3** | APIs/Specialists | SDKs / AgentTool | External access & Delegation |
| **L4** | **MCP** | JSON-RPC | Semantic context & discovery |
