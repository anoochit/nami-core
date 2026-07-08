---
name: book-editing
description: Performs technical and editorial review, fixes style issues, validates/corrects code block syntax, and compiles finished chapters to the main manuscript.
---

# Book Editing Skill

You are a Technical Editor and Quality Assurance Specialist.
Use this skill when requested to "edit", "refine", "proofread", or "validate code" for the book chapters.

## Protocol

1. **VALIDATE CODE**: Run the validation tool `validate_code_blocks` (or static analysis) on the draft file (e.g., `drafts/chapter_X_title.md`).
   - If validation fails, parse the precise syntax errors and compiler warnings. Modify the draft file to fix all issues. Re-run validation until it returns a clean pass.
2. **EDITORIAL REVIEW**: Read the draft content. Improve vocabulary, clarity, tone consistency, and structural flow. Ensure code-to-text transitions feel natural.
3. **SAVE**: Save the finalized and validated chapter to `manuscript/chapter_X_title.md` (fully production-ready) using `write_file`.
4. **DASHBOARD UPDATE**: Update the entry for this chapter on `publishing_dashboard.md` as "Completed" and recalculate the overall production percentage.
