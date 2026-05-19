# Component Reference: Core Modules

This document provides a detailed breakdown of the primary components within the Nami framework.

## 🏃 Agent Runner (`src/runner.rs`)

The `AgentRunner` is the main orchestration struct used across all modes to execute agent tasks.

-   **Persistence**: Integrates with `adk-session` and `adk-memory` to maintain state across restarts.
-   **Retry Strategy**: Implements an exponential backoff loop for transient errors (categorized in `src/utils/mod.rs`).
-   **Event Compaction**: Automatically summarizes conversation history when the context window limits are approached, using the `EventsCompactionConfig`.
-   **Streaming**: Provides a clean interface for handling asynchronous event streams from the ADK runner.

## 🧠 The Agent (`src/agent/agent.rs`)

The `Agent` is the core identity of the system, configured via `build_agent()`.

-   **Persona Loading**: Dynamically constructs the system prompt by reading:
    -   `AGENT.md`: Identity and tone.
    -   `USER.md`: User profile and preferences.
    -   `MEMORIES.md`: Synthesized long-term facts.
    -   `STATE_PROTOCOL.md`: Operating procedures.
-   **Skill Discovery**: Automatically loads "Skills" (standalone agent logic) from the `workspace/` directory using `with_skills_from_root`.
-   **Specialists**: Manages a set of specialized sub-agents (e.g., Coder, Researcher) that can be invoked for specific tasks.

## 🛠 Dependencies (`src/modes/startup.rs`)

The `Dependencies` struct holds shared service instances used throughout the application.

-   **Session Service**: A SQLite-backed service for storing conversation history and agent state.
-   **Memory Service**: A SQLite-backed vector/semantic memory service for long-term fact retrieval.
-   **Memory Adapter**: Bridges the specific memory service to the general `adk-rust::Memory` trait used by the tools.

## ✨ Reflection Service (`src/agent/reflection.rs`)

A background service that acts as a "Memory Architect."

-   **Observation**: Periodically reads recent session logs.
-   **Synthesis**: Uses a specific reflection model to extract new learnings, facts, and corrections.
-   **Updates**: Automatically updates the `MEMORIES.md` file and the searchable memory database, ensuring Nami grows smarter over time.

## 📅 Scheduler (`src/modes/scheduler.rs`)

A background loop that enables autonomous agent actions.

-   **Cron Support**: Executes tasks based on defined schedules.
-   **Self-Correction**: Uses the agent's logic to determine if a scheduled task was successful or requires follow-up.
