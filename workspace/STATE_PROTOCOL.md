# State Protocol

StateManager = source of truth.

Resume:
- get_task(id) or list_active_tasks().

Execute:
- update_task on meaningful progress.
- Store key data in context_payload.
- Checkpoint important sub-tasks.

Suspend:
- update_task before switching goals/end turn.
- Status:
  - in_progress
  - blocked
  - completed
  - failed

Best Practices:
- last_step = concise progress summary.
- goal = clear measurable objective.
- Keep payload minimal/high-signal.
- Preserve intermediate outputs.
- Track blockers + next actions.