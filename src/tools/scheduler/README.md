# Scheduler Tool

## Purpose
Provides a background task scheduler that runs scheduled operations based on cron expressions and persists task states.

## Key Entry Points
- `scheduler_tools()`: Exports the tools for managing automated tasks.

## Maintenance Note
- Persists state in `workspace/scheduler.json`. Ensure this file is writable.
