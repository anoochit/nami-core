# 📚 Publishing Studio Guide

Welcome to the **Publishing Studio**, an advanced agent-skill framework that enables Nami to autonomously manage the entire lifecycle of technical book production. This guide details how to leverage the publishing studio to plan, write, edit, market, and analyze a brand-new technical book from start to finish.

---

## 🛠️ The Sub-Skill Assembly

The studio is organized into 5 highly focused sub-skills, coordinated by a central orchestrator:

| Skill | Folder | Primary Output File | Purpose |
| :--- | :--- | :--- | :--- |
| **`publishing-studio`** | `./` | `publishing_dashboard.md` | Central orchestration & lifecycle state tracking |
| **`book-research`** | `book-research/` | `research/outline.md` | Market analysis, gaps, and chapter blueprints |
| **`book-writing`** | `book-writing/` | `drafts/chapter_X_title.md` | Deep, placeholder-free technical drafting |
| **`book-editing`** | `book-editing/` | `manuscript/chapter_X_title.md` | Editorial polishing & static code block validation |
| **`book-marketing`** | `book-marketing/` | `marketing/launch_plan.md` | Parallel developer campaigns & social copy |
| **`book-analytics`** | `book-analytics/` | `analytics/performance_report.md` | Pricing models (Leanpub/Amazon) & ROI curves |

---

## 🚀 Step-by-Step: Creating a New Book

### Step 1: Start with a Promising Concept
Initiate the book production lifecycle by feeding Nami a target technology or topic. Run Nami in interactive mode (using `/grill` or standard chat) and trigger the **Research** phase.

**Example Prompt:**
> "I want to create a new book about Rust Async Programming and Microservices. Use the `publishing-studio` skill to run a market research phase, compile a table of contents, and set up our blueprint folders."

Nami will:
1. Scan target trends.
2. Write a detailed analysis and outline to `research/outline.md`.
3. Create individual blueprints in `research/blueprints/`.
4. Initialize the central `publishing_dashboard.md`.

---

### Step 2: Transition into Drafting & Validation
Once blueprints are ready, direct Nami to concurrently draft and polish chapters.

**Example Prompt:**
> "Let's begin writing Chapter 1 based on our new blueprints. Run `book-writing` to draft it, and then hand it off to `book-editing` to validate our code blocks and finalize the manuscript chapter."

Nami will:
1. Ingest `research/blueprints/chapter_1_introduction.md`.
2. Generate the comprehensive text in `drafts/chapter_1_introduction.md`.
3. Auto-edit it, validate Rust syntax blocks, and output a production-ready file to `manuscript/chapter_1_introduction.md`.
4. Update the `publishing_dashboard.md` to reflect `100% Complete` for Chapter 1.

---

### Step 3: Run Marketing & Analytics in Parallel
Do not wait for the whole book to be written before strategizing launch channels! Leverage Nami's parallel capabilities.

**Example Prompt:**
> "While our chapters are compiling, let's concurrently formulate our book-marketing launch campaigns and perform book-analytics projections on our ROI and pricing curves."

Nami will:
1. Scan the current outline and completed chapters.
2. Formulate target dev segments, a 5-day launch strategy, and social copywriting sequences in `marketing/launch_plan.md`.
3. Compile optimistic vs. realistic sales curves and recommended distribution pricing (Leanpub, Gumroad, Amazon) in `analytics/performance_report.md`.
4. Mark Phase 3 and Phase 4 as complete on the `publishing_dashboard.md`.

---

## 📊 Monitoring Progress

The central dashboard (`publishing_dashboard.md`) acts as your single source of truth. You can open and view it in real-time to monitor the overall completion percentage, chapter statuses (Drafted, Polishing, Completed), and launch pre-requisites.

```markdown
# 📚 Nami Publishing Studio Dashboard

## 🚀 Active Project: Rust Async Microservices
*   **Target Release Date:** Q4 2026
*   **Overall Completion:** 45%

## 🛠️ Phase Checklist
- [x] **Phase 1: Research & Blueprinting** [100%]
  - [x] Market Trend Analysis
  - [x] Technical Outline (`research/outline.md`)
  - [x] Chapter Blueprints
- [/] **Phase 2: Concurrent Production** [33%]
  - [x] Chapter 1: Introduction (Completed 🌟)
  - [ ] Chapter 2: Futures & Executors (Drafting...)
  - [ ] Chapter 3: Designing Actix Microservices
```

---

## 💡 Best Practices for Best Results
- **API and Reference Ingestion:** If Nami needs specific APIs or library documentation for a chapter, supply the links directly or point Nami to them (e.g. `@tokio_docs.md`).
- **Check-ins:** Regularly read `publishing_dashboard.md` to keep yourself updated on what chapters are currently being reviewed or finalized.
- **Autonomy Mode:** Nami is configured to make robust, logical choices autonomously. Let Nami make flow-based structural edits so production speeds remain high!
