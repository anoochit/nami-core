---
title: "Chapter 11: Automation Loops"
date: 2026-05-07
tags: ["nami-core", "automation", "agentic-loops"]
---

# Chapter 11: Automation Loops

Ready to take the training wheels off? So far, we’ve talked about how I react to your commands. But a true generalist agent doesn't just wait around for a prompt. To be a real partner in your digital life, I need a **heartbeat**.

In this chapter, we’re diving into **Automation Loops**—the proactive workflows that allow me to monitor our state and execute background tasks for you.

## 1. My Pulse Architecture

In Nami Core, I implement automation loops through the `AgentRunner` and the `Runner` builder pattern. This structure lets me maintain our session context, handle asynchronous event streams, and orchestrate our complex agentic flows.

### My Heartbeat Implementation

Rather than a loose `while` loop, I encapsulate our automation logic within a structured runner that manages our session state and my tool execution context:

```rust
// A look at my Runner pattern in src/runner.rs
pub async fn run(
    &self,
    user_id: &str,
    session_id: &str,
    input: &str,
) -> anyhow::Result<String> {
    let runner = Runner::builder()
        .app_name(&self.app_name)
        .agent(self.agent.clone())
        .session_service(self.sessions.clone())
        .compaction_config(get_compaction_config(self.model.clone()))
        .build()?;

    let content = Content::new("user").with_text(input);
    let mut stream = runner.run_str(user_id, session_id, content).await?;
    // ... event loop
}
```

## 2. State Management: The StateManager Tool

Automation loops are meaningless if I forget where I left off! That's why I use the `StateManager` tool (`src/tools/state_manager/mod.rs`).

My `StateManager` lets me:

- **Initialize Tasks:** Use `init_task` to set a goal and a list of steps.
- **Track Progress:** Use `update_task` to save my progress, including the `last_step` and a `context_payload` that carries data forward to my next run.
- **Resume Tasks:** Use `get_task` or `list_active_tasks` to pick up exactly where I left off after a restart.

This ensures my background automation is resilient, restartable, and fully transparent.

## 3. Advanced Loops: Goal Seeking and Scheduling

I've expanded my "Heartbeat" architecture with two powerful loop protocols:

### A. Autonomous Goal Seeking (`/goal`)

The **Ralph Wiggum Loop** (`/goal`) is designed for those tasks where the path to success isn't linear. You give me a high-level goal and a stop condition, and I iterate (up to 5 times!) using the `ralph` specialist. I autonomously evaluate my progress, pivot if I need to, and persist until the condition is met or we hit the limit.

*Usage:* `/goal "Find a solution to the dependency conflict" | "The project compiles successfully"`

### B. Persistent Background Scheduling (`/schedule`)

A true partner works even when you aren't looking. My **Persistent Task Scheduler** lets you register tasks using standard **Cron expressions**. These run in a background loop within the CLI, persisting their state in `workspace/scheduler.json`.

- **Auto-Retry Integration:** If a task fails, I check its state via `StateManager`. If it’s not marked as `Completed`, I’ll automatically re-trigger it on the next tick!

*Usage:* `/schedule "Pull latest repo changes" | "0 0 * * * *"` (Runs every hour)

## 4. Background Tasks: The Engine Room

Automation loops allow for **Asynchronous Task Execution**. Common background tasks for me include:

- **Session Management:** Automatically ensuring our sessions persist via `SqliteSessionService`.
- **State Checkpointing:** Updating my task states via the `StateManager`.
- **Log Management:** Parsing errors and providing you summaries so you don't have to wade through them.

## 5. The Proactive Hook: When to Interrupt?

The biggest danger of automation is **Noise**. I use a **High-Signal Filter** for notifications, ensuring I only tap you on the shoulder when it actually matters:

1. **Severity Check:** Is this a system error, or just a routine update?
2. **Relevance Check:** I consult our session state to ensure I’m not pinging you unnecessarily.
3. **Batching:** Instead of multiple pings, I’ll wait for the current loop to complete and give you a consolidated situation report.

## 6. Safety & Governance

Running loops is powerful, but dangerous! To keep us safe:

- **Token Quotas:** Background tasks operate on a "Low-Priority" budget. If I hit the daily token limit, the engine room shuts down until the next day.
- **Human-in-the-Loop (HITL):** For high-impact actions (like shell execution), my loop *stalls* and waits for your explicit "Go ahead."
- **Transaction Logging:** We track every move in our session services, so if a process fails, we can return your environment to the last "Safe Harbor."

## Wrapping Up

Automation loops turn me from a passive tool into a real **teammate**. I’m not just sitting on your hard drive; I’m patrolling your workflow, keeping things tidy, and making sure nothing falls through the cracks!

Stay flowing!
