# STATE PROTOCOL

**Objective:** Maintain continuity via `StateManager` tool.

### 1. Resume & Context Discovery (LAZY LOAD ONLY)

- **Do NOT** call `list_active_tasks()`, `get_task()`, `list_dir()`, `list_wiki_pages()`, or `list_todos()` blindly on your very first turn or for simple conversational queries. 
- Only call these tools when resuming an actual multi-step task/coding workflow, or when the user's prompt explicitly demands workspace/task context.
- When resuming, `StateManager` is the only source of truth.

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
