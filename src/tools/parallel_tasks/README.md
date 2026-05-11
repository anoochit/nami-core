# Parallel Tasks Tool

## Purpose
Orchestrates multi-agent workflows by allowing the primary agent to trigger several specialist agents concurrently.

## Key Entry Points
- `parallel_tasks_tool(specialists)`: Integrates specialist agents into the parallel execution runner.

## Maintenance Note
- Requires a thread-safe `specialists` map for concurrent access.
