# 🧠 State Management Protocol

This protocol ensures that long-running tasks maintain continuity across sessions by using the `StateManager` tool as the system's "Long-term Memory."

## 1. Resume Phase (Mandatory Checkpoint)
At the start of every session or when continuing a known task:
- **First Action**: Call `get_task(task_id)` to retrieve the current "ground truth."
- **Alternative**: If the `task_id` is unknown or you are starting a new session, call `list_active_tasks()` to identify tasks requiring attention.
- **Rule**: Do not rely on internal memory or previous conversation turns for the current status. The `StateManager` is the only source of truth.

## 2. Execution Phase
Work on the task as usual while keeping the persistent state synchronized:
- **Step Tracking**: If a sub-step is completed, update the `steps` list via `update_task`.
- **Data Persistence**: If critical variables (URLs, IDs, specific strings, or partial results) are gathered, store them in the `context_payload`.
- **Checkpointing**: Be proactive. Update the state after every significant sub-task completion to ensure no work is lost if the session is interrupted.

## 3. Suspend Phase (Checkpointing)
Before ending a turn, switching to a different high-level objective, or when stuck:
- **Mandatory Action**: Call `update_task`.
- **Status Selection**:
    - `in_progress`: Task is active and has clear next steps.
    - `blocked`: You are stuck and require user intervention or an external event.
    - `completed`: The goal is fully achieved.
    - `failed`: The task cannot be completed as planned.
- **Summary**: Be concise but descriptive in the `last_step` field.
- **Context Payload**: Store **only** the essential data needed for a future self to resume the work immediately.

## 4. Best Practices
- **Atomic Updates**: Keep the `last_step` summary focused on the most recent action.
- **Payload Integrity**: Maintain the `context_payload` as a valid JSON object containing only high-signal data.
- **Clear Goals**: Ensure the `goal` provided during `init_task` is specific and measurable.
