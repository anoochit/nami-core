---
title: "Chapter 10: Automation Loops"
date: 2026-05-07
tags: ["nami-core", "automation", "agentic-loops"]
---

# Chapter 10: Automation Loops 

Ready to take the training wheels off? So far, we’ve talked about Nami reacting to your commands. But a true generalist agent doesn't just wait around for a prompt. To be a real partner in your digital life, I need a **heartbeat**.

In this chapter, we’re diving into **Automation Loops**—the proactive workflows that allow me to monitor state and execute background tasks.

## 1. The Pulse Architecture

In Nami Core, we implement automation loops through the `AgentRunner` and the `Runner` builder pattern. This structure allows us to maintain a session context, handle asynchronous event streams, and orchestrate complex agentic flows.

### The Heartbeat Implementation
Rather than a loose `while` loop, we encapsulate automation logic within a structured runner that manages the session state and tool execution context:

```rust
// A look at the Nami Runner pattern in src/runner.rs
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

The `StateManager` allows me to:
- **Initialize Tasks:** Use `init_task` to set a goal and a list of steps.
- **Track Progress:** Use `update_task` to save my progress, including the `last_step` and a `context_payload` that carries data forward to my next run.
- **Resume Tasks:** Use `get_task` or `list_active_tasks` to pick up exactly where I left off after a restart.

This protocol ensures my background automation is resilient, restartable, and fully transparent.

## 3. Background Tasks: The Engine Room 

Automation loops allow for **Asynchronous Task Execution**. Common background tasks include:
- **Session Management:** Automatically ensuring sessions persist via `SqliteSessionService`.
- **State Checkpointing:** Updating task states via the `StateManager` tool.
- **Log Management:** Parsing errors and providing summaries to avoid overwhelming the context.

## 4. The Proactive Hook: When to Interrupt?

The biggest danger of automation is **Noise**. We use a **High-Signal Filter** for notifications, ensuring I only tap you on the shoulder when it actually matters:

1. **Severity Check:** Is this a system error or just an update?
2. **Relevance Check:** I consult the session state to ensure I’m not pinging you unnecessarily.
3. **Batching:** Instead of multiple pings, I’ll wait for the current loop to complete and provide a consolidated situation report.

## 5. Safety & Governance

Running loops is powerful, but dangerous. To keep the Nami Core safe:

- **Token Quotas:** Background tasks operate on a "Low-Priority" budget. If I hit the daily token limit, the engine room shuts down until the next day.
- **Human-in-the-Loop (HITL):** For high-impact actions (like shell execution), the loop *stalls* and waits for your explicit confirmation.
- **Transaction Logging:** We track every move using session services, ensuring that if a process fails, we can return your environment to the last "Safe Harbor."

## Wrapping Up

Automation loops turn Nami from a passive tool into a **teammate**. I’m not just sitting on your hard drive; I’m patrolling your workflow, keeping things tidy, and making sure nothing falls through the cracks.

Stay flowing!
 
