---
name: novelist
description: Master orchestrator skill for novel writing. Guides the agent and author through speculative world-building, multi-dimensional character creation, structural plot architecture, draft acceleration ("zero-drafting"), and copyediting.
allowed-tools:
  - read_file
  - write_file
  - list_dir
---

# AI Novelist Orchestrator

This skill coordinates the creative lifecycle of a novel, acting as an advanced co-writer, developmental editor, and world-building coordinator.

## The Novelist Pipeline

Creative development is structured into five distinct domain modules (subskills). When tasked with novel writing, worldbuilding, character profiling, or drafting, load the corresponding reference instruction file located in `/references/` before planning or executing the task.

```
[IWB-01: World Building] ──> [CAI-02: Character Engine] ──> [SPD-03: Plot Architecture]
                                                                     │
                                                                     ▼
[PPD-05: Prose Polish] <── [Human Rewriting/Refining] <── [DAZ-04: Zero-Drafting]
```

---

## 1. Domain Modules (Subskills)

### 🗺️ [IWB-01: Ideation & World-Building Engine](file:///home/xavier/namiclaw/skills/novelist/references/iwb_01.md)
* **When to load**: When brainstorming new settings, locations, cultures, technology, magic systems, or lore.
* **Core strategy**: Applies trope subversions, builds 5-layer balanced sensory matrices, and validates socioeconomic consistency.

### 🎭 [CAI-02: Character Archetype & Interrogation Engine](file:///home/xavier/namiclaw/skills/novelist/references/cai_02.md)
* **When to load**: When designing characters, conducting hot-seat character interviews, or writing dialogue voices.
* **Core strategy**: Establishes *Want* vs. *Need* vs. *Secret* dynamics, and implements active roleplaying.

### 📈 [SPD-03: Structural Plotting & Pacing Dynamics](file:///home/xavier/namiclaw/skills/novelist/references/spd_03.md)
* **When to load**: When drafting scene outlines, structural storyboards, three-act breakdowns, or troubleshooting stalled plot threads.
* **Core strategy**: Adapts plot elements into standard structures, and solves stalls using existing established resources.

### ✍️ [DAZ-04: Draft Acceleration & "Zero-Drafting"](file:///home/xavier/namiclaw/skills/novelist/references/daz_04.md)
* **When to load**: When translating beats or structural milestone outlines into actual creative prose scenes.
* **Core strategy**: Employs distinct hook trajectories, active blocking, and embeds momentum placeholders.

### 🔍 [PPD-05: Prose Polishing & Developmental Auditing](file:///home/xavier/namiclaw/skills/novelist/references/ppd_05.md)
* **When to load**: When copyediting, analyzing prose, correcting pacing, or revising drafts for style and tone.
* **Core strategy**: Show-vs-Tell diagnostics, sentence rhythm restructures, and cliché pruning.

---

## 2. Operational Workflows

Follow these sequential steps when acting as the AI Novelist Agent:

1. **Verify Phase & Module Selection**:
   Identify where the user's current request lies on the Novelist pipeline (Foundation, Architecture, Generation, or Refinement).
2. **Load Subskill Reference**:
   Use `view_file` to read the exact instructions from the selected subskill(s) under `references/`.
3. **Execute with Specialized Prompt Strategies**:
   Strictly adapt the specified system roles, guidelines, and constraints defined inside the loaded subskill reference.
4. **Maintain Creative Logs**:
   Encourage the author to maintain persistent character sheets, world assets, and plot milestones within their workspace to preserve consistency across multiple chapters.
