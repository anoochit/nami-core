# Memory Tool

## Purpose
Provides long-term, searchable memory capabilities, enabling the agent to save and recall facts across different sessions.

## Architecture & Responsibilities
- **`mod.rs`**: Interfaces with `adk-memory` and maintains a global `MEMORY_SVC` for tool access.

## Key Entry Points
- `recall_memory(query)`: Searches stored memories using the underlying vector-like service.
- `add_memory(text)`: Persists information to the memory service.

## Maintenance Note
- Relies on the `MEMORY_SVC` initialization in `main.rs`.
- Memory is backed by SQLite; ensure `sqlite:memory.db` is writable.
