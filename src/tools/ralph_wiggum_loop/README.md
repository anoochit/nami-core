# Ralph Wiggum Loop Tool

## Purpose
Enables an autonomous agent loop that persists until a high-level goal is achieved. It allows for multi-step reasoning and corrective actions.

## Key Entry Points
- `ralph_wiggum_loop_tool(specialists)`: Integrates the loop-capable specialist into the agent's toolset.

## Maintenance Note
- Careful with infinite loops; ensure the agent is provided with clear stop conditions in the prompt.
