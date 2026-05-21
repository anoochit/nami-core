# STATE PROTOCOL

**Objective:** Maintain continuity via `StateManager` tool.

### 1. Resume

- Call `get_task(id)` or `list_active_tasks()` first.
- StateManager = Only source of truth.

### 2. Execute

- `update_task` on step completion.
- Store critical data in `context_payload`.
- Checkpoint after every significant sub-task.

### 3. Suspend

- Call `update_task` before turn end/switching goals.
- **Status:** `backlog`, `todo`, `in_progress`, `in_review`, `blocked`, `done`, `cancelled`.
- **Payload:** Minimal/High-signal JSON only.

### 4. Best Practices

- `last_step` = summary of last action.
- Clear/measurable `goal` in `init_task`.
