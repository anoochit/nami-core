# Development Plan: Structured State Management for Long-Running Processes

## 1. Objective
Transition from manual Markdown logging (`TaskLog.md`) to a structured, tool-based state management system. This ensures session continuity, prevents log hallucinations, and allows for programmatic validation of agent progress.

## 2. Proposed Architecture

### 2.1 Storage
- **Location**: `workspace/task_states.json`
- **Format**: JSON array of task objects (consistent with `todos.json`).

### 2.2 Data Schema
| Field | Type | Description |
| :--- | :--- | :--- |
| `task_id` | String | Unique identifier (UUID or descriptive slug). |
| `status` | Enum | `in_progress`, `blocked`, `completed`, `failed`. |
| `goal` | String | High-level objective of the task. |
| `steps` | List<Step> | List of execution steps with status (from `TaskLog.md`). |
| `last_step` | String | Summary of the last completed action. |
| `context_payload` | JSON | Critical variables (URLs, IDs, data) for resumption. |
| `updated_at` | DateTime | ISO 8601 timestamp of last update. |

### 2.3 Toolset (`StateManager`)
- `init_task(task_id, goal, steps)`: Initialize a new long-running task.
- `update_task(task_id, status, last_step, context_payload, updated_steps)`: Update task progress.
- `get_task(task_id)`: Retrieve "ground truth" for a specific task.
- `list_active_tasks()`: List all tasks with `in_progress` or `blocked` status.

## 3. Implementation Steps

### Phase 1: Tool Scaffolding
1. Create `src/tools/state_manager/mod.rs`.
2. Define `TaskState`, `Step`, and argument structs.
3. Implement `load_states()` and `save_states()` helper functions.

### Phase 2: Tool Implementation
1. Implement `init_task` with validation (ensure `task_id` uniqueness).
2. Implement `update_task` with partial update support.
3. Implement `get_task` and `list_active_tasks`.

### Phase 3: Registration & Integration
1. Register `state_manager` in `src/tools/mod.rs`.
2. Update the main agent registry to include the new tools.

### Phase 4: Protocol & Documentation
1. Update Agent System Instructions (as per `IDEA_LONGRUN.md`).
2. Add a `STATE_PROTOCOL.md` in the workspace to guide agent behavior.
3. Create a migration guide for moving from `TaskLog.md` to the tool.

## 4. Verification Plan
- **Unit Tests**: Test serialization/deserialization and state transitions.
- **Integration Test**: Simulate a multi-session task where the agent saves state and resumes.
- **Persistence Check**: Verify `task_states.json` is correctly updated in the `workspace/` directory.
