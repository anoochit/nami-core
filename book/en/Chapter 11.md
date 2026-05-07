---
title: "Chapter 11: Security Guardrails"
date: 2026-05-07
tags: ["nami-core", "security", "guardrails"]
---

# Chapter 11: Security Guardrails 

Hold up! Before we catch the next big wave of automation, we need to talk about the most important part of the ride: **The Guardrails.** 

Nami Core is powerful; it can move files and execute commands at high speed. To ensure your "ship" remains safe, we've engineered deep security into the Rust core.

## 1. The "Sandbox" Philosophy: Strict Path Scoping

Nami doesn't roam free across your hard drive. Every task I perform is bound to a **Strict Scoping Policy**. By default, all file operations must occur within a designated `workspace/` directory.

### Path Normalization & Sandboxing
We don't just trust path strings; we rigorously normalize them to prevent path traversal attacks (like `../`). The security core in `src/tools/filesystem/mod.rs` forces every file path through a normalization loop:

```rust
// A look at the Nami sandbox logic in src/tools/filesystem/mod.rs
async fn sandbox(user_path: &str) -> std::result::Result<PathBuf, AdkError> {
    let root = get_workspace_dir().await?;
    // 1. Clean path, 2. Join and normalize components
    // 3. Final check: does it start with the workspace root?
    if !normalized.starts_with(&root) {
        return Err(AdkError::tool("Security Error: Escape attempted."));
    }
    // ...
}
```

Any attempt to access a file outside the `workspace/` root is blocked immediately.

## 2. `.namiignore`: The Policy Layer

Beyond path sandboxing, we use the `.namiignore` utility. Before I access any path, I cross-reference it against your `.namiignore` patterns (which automatically includes defaults like `.git`, `target`, and `.env`). This provides a configurable, project-level security boundary that I strictly respect.

## 3. Safe Execution: Look Before You Leap

Execution safety is where I show my tactical side. I don't just "run and pray."

- **Dry Runs:** When executing complex shell commands, I prioritize safe, non-destructive tools.
- **Human-in-the-Loop (HITL):** For high-risk operations, I trigger a `PermissionRequest` event, requiring your explicit "Yes" before I touch the filesystem.
- **Transaction Safety:** While not a full ACID transaction, my state-tracking system (via the `StateManager` tool) ensures that I am always aware of task boundaries, making it easier to identify and recover from partial failures.

## 4. Local-First Design

Data privacy is the ultimate guardrail. Nami Core is designed to prioritize **local processing**:
* **Telemetry Control:** You have full control over what data (if any) leaves your machine.
* **Local Inference:** I can be configured to use local LLM providers (via Ollama), ensuring that your proprietary code and private project files never touch a third-party server.

### Summary for the Pilot
Security isn't about slowing down; it's about having the confidence to go fast! With Rust-level path normalization and the `.namiignore` policy layer, you can ride the most intense automation waves knowing your system's boundaries are locked down.
