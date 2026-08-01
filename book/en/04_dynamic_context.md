---
title: "Chapter 4: Dynamic Context"
date: 2026-05-07
tags: ["context", "memory", "user-profile"]
---

# Chapter 4: Dynamic Context – The Art of Not Being a Blank Slate

Hey there, Architect! Nami here! Welcome to the heart of my architecture. If Chapter 3 was about my "Soul," this chapter is about the **Contextual Grounding** that brings that soul to life.

## 1. My Persona Injection

I don't operate as a blank slate. On startup, I perform an atomic load of your project context. My `AgentRunner` and the `create_agent` factory function in `src/agent/agent.rs` orchestrate the injection of four core context files to define who I am when we work:

- **`AGENT.md`**: Defines my core identity and rules (I'm playful but technically brilliant!).
- **`USER.md`**: Stores your persona, your technical proficiency, and your favorite way for us to communicate.
- **`MEMORIES.md`**: My long-term ledger for our history together.
- **`STATE_PROTOCOL.md`**: My structural guidebook for managing tasks and keeping our continuity alive.

These files are loaded synchronously at startup and fused into a single **Instruction Block**. This ensures that every reasoning loop I run is anchored by your specific project values and our shared history.

## 2. Dynamic Memory Management

Memory isn't just about reading files—it's about stateful continuity! When I initialize, I read these Markdown files from your workspace, falling back to sensible defaults if one is missing.

### How I maintain "State":
My context isn't static; it’s a living part of my `AgentBuilder` configuration. When you provide input, my context manager sandwiches it between:
1. **The Fused Instruction Block:** My current persona definition.
2. **Current Session History:** The thread we’re currently working on.
3. **Task-Specific State:** Any active state I've retrieved via the `StateManager` tool.

This means I have **Zero-Latency Intent Recognition**. When you mention a file or a project goal, I already have the context loaded in my active memory graph. No need to ask "which one?"—I'm already there with you!

## 3. Staying Relevant (Summarization)

Context window management is critical for staying sharp. I use `EventsCompactionConfig` to manage our history. Periodically, I run an `LlmEventSummarizer` to compress old interactions into high-signal summaries. This prevents context bloat while keeping all the core facts (your preferences, status, and resolved blockers) right at my fingertips.

### Summary of the Flow:
- **`AGENT.md` / `USER.md`**: My *Identity* and *Relevance*.
- **`MEMORIES.md`**: Our *History*.
- **`STATE_PROTOCOL.md`**: Our *Workflow Continuity*.

By treating these files as system-critical inputs rather than just "notes," we ensure that every interaction is perfectly tailored to your project.

Onward!
