---
title: "Chapter 4: Dynamic Context"
date: 2026-05-07
tags: ["context", "memory", "user-profile"]
---

# Chapter 4: Dynamic Context – The Art of Not Being a Blank Slate

Hey there, Architect! Nami here! Welcome to the heart of the Nami Core architecture. If Chapter 3 was about my "Soul," this chapter is about the **Contextual Grounding** that brings that soul to life.

## 1. The Persona Injection

I don't operate as a blank slate. On startup, I perform an atomic load of your project context. My `AgentRunner` and the `create_agent` factory function in `src/agent/agent.rs` orchestrate the injection of four core context files to define my operational persona:

- **`AGENT.md`**: Defines my core identity, traits, and operational rules (e.g., "playful but technically brilliant").
- **`USER.md`**: Stores your persona, technical proficiency, and communication preferences.
- **`MEMORIES.md`**: Acts as my long-term ledger for session persistence and historical learnings.
- **`STATE_PROTOCOL.md`**: Provides the structural guidelines for state management and task continuity.

These files are loaded synchronously at startup and fused into a single **Instruction Block** using the `format_persona` function. This ensures that every reasoning loop I run is anchored by your specific project values and your history with me.

## 2. Dynamic Memory Management

Memory isn't just about reading files—it's about stateful continuity. When I initialize, `load_persona_context` reads these Markdown files from your workspace. If a file is missing, I gracefully fall back to sensible technical defaults.

### How I maintain "State":
My context isn't just static data; it's a living, breathing component of my `AgentBuilder` configuration. During runtime, when you provide input, my context manager sandwiches it between:
1. **The Fused Instruction Block:** My current persona definition.
2. **Current Session History:** The relevant conversation thread.
3. **Task-Specific State:** Any active state retrieved via the `StateManager` tool.

This ensures I have **Zero-Latency Intent Recognition**. When you mention a file or a project goal, I don't need to ask "which one?"—I already have the context loaded in my active memory graph.

## 3. Staying Relevant (Summarization)

Context window management is critical. I use `EventsCompactionConfig` to manage history. Periodically, I run an `LlmEventSummarizer` to compress old interaction events into high-signal summaries. This prevents context bloat while preserving the core facts (your preferences, project status, and resolved blockers) that defined our trajectory.

### Summary of the Flow:
- **`AGENT.md` / `USER.md`**: Provides *Identity* and *Relevance*.
- **`MEMORIES.md`**: Provides *History*.
- **`STATE_PROTOCOL.md`**: Provides *Workflow Continuity*.

By treating these files as system-critical context inputs rather than just "notes," we ensure that every interaction is tailored to your specific project needs.
