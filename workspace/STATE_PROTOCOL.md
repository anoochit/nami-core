# State Management Protocol

This protocol ensures that long-running tasks maintain continuity across sessions by using the `StateManager` tool as a "Long-term Memory."

## 1. Resume Phase (Mandatory Checkpoint)
At the start of every session or when continuing a known task:
- **First Action**: Call `get_task(task_id)` to retrieve the current "ground truth."
- **Alternative**: If the `task_id` is unknown, call `list_active_tasks()` to see what needs attention.
- **Rule**: Do not rely on internal memory or previous conversation turns for the current status. Use the tool.

## 2. Execution Phase
Work on the task as usual.
- If a sub-step is completed, update its status.
- If critical data (URLs, IDs, specific strings) is gathered, store it in the `context_payload`.
- Be proactive in checkpointing significant progress.

## 3. Suspend Phase (Save Progress)
Before ending a turn or moving to a different high-level task:
- **Mandatory Action**: Call `update_task`.
- **Status Selection**:
    - `in_progress`: Task is active and has clear next steps.
    - `blocked`: You are stuck and need user intervention or external events.
    - `completed`: The goal is fully achieved.
    - `failed`: The task cannot be completed.
- **Summary**: Be concise in `last_step`.
- **Context**: Store **only** what is needed to resume (e.g., "Last processed item index: 42").

## 4. Transitioning from TaskLog.md
- **New Tasks**: Always use `init_task`.
- **Legacy Tasks**: For tasks currently in `TaskLog.md`, initialize them via the tool and move the current steps/status into the tool's state. You can then retire the `.md` file for that specific task.
