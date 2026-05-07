---
title: "Chapter 2: AI-Native Engineering"
date: 2026-05-07
tags: ["ai-native", "engineering", "reasoning-loop"]
---

# Chapter 2: AI-Native Engineering

## Beyond the Wrapper
In the early days of LLM integration, software was designed as a "wrapper" around a prompt. AI-Native Engineering is a paradigm shift where the LLM is not just an added feature, but the central reasoning engine of the system.

### The Reasoning Loop
Traditional software follows rigid if-then-else logic. AI-native systems use a reasoning loop:
1. **Perception:** Reading the user's intent and the current environment (Wiki, Files, System).
2. **Planning:** Breaking down a complex goal into executable tool calls.
3. **Execution:** Using specialized tools (MCP, Shell, Web Search) to interact with the world.
4. **Reflection:** Assessing the results of actions and adjusting the plan dynamically.

### Designing for Uncertainty
AI-native systems must be built to handle non-deterministic outputs. This requires:
- **Strict Schemas:** Using JSON and structured definitions for tool interactions.
- **Context Management:** Pruning and prioritizing information to fit within context windows.
- **Verifiability:** Building tools that allow the agent to verify its own work (e.g., checking if a file was actually written correctly).

### Nami's Approach
Nami is built on these principles, using the Wiki vault as a living, breathing context that the engine can query and update in real-time, moving beyond static data structures.
