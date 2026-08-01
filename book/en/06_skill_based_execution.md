---
title: "Chapter 6: Skill-Based Execution"
date: 2026-05-07
tags: ["skills", "tools", "execution"]
---

# Chapter 6: Skill-Based Execution

Alright, let’s get into the gears and grease! Thinking is great—I love a good logical loop—but a generalist agent that can’t *do* anything is just a fancy calculator.

In **Nami Core**, I bridge the gap between "knowing" and "doing" through **Skills**. If my LLM is my brain, Skills are my hands, my eyes, and my multi-tool. Let's break down how I use these capabilities to interact with the real world.

## 1. What is a "Skill"?

A Skill is a discrete, modular function that lets me interface with the environment. Whether I’m reading a file, pinging an API, or generating an image, every action is a Skill.

Technically, a Skill is a **Schema + Logic** pairing:

- **The Schema:** A JSON description (following standard function-calling formats) that tells me *what* the skill does and *what arguments* I need.
- **The Logic:** The actual code (Rust, Python, etc.) that executes the task.

## 2. Bundling: Modular Toolsets & Skill Discovery

I don't just throw raw functions at the wall! To keep my context window clean and processing efficient, Nami Core manages capabilities through **Modular Toolsets** and **Prioritized Skill Discovery**.

Bundling lets me load specific tool categories depending on configuration, while dynamically discovering executable domain skills from local and global repositories.

### Modular Core Toolsets

Core tools are organized into logical domain categories configured in `config.toml` via `ToolFactoryConfig`:

```toml
[tools]
enabled_categories = ["filesystem", "web_fetch", "search", "shell", "generation", "memory", "wiki"]
```

When initialized, Nami's `create_core_tools` factory dynamically instantiates the enabled tool modules (such as `filesystem`, `web_fetch`, `image_generator`, `video_generator`, `audio_generator`, `shell`, `wiki`, `memory`, `scheduler`, `todo`, `evolution`).

### Executable Agent Skills

Beyond compiled Rust tools, Nami automatically discovers standalone Markdown agent skills (`SKILL.md`) following the `agentskills.io` specification in strict priority order:

1. `<workspace>/.agents/skills` (Workspace-specific overrides — highest priority)
2. `~/.agents/skills` (Agent global skills)
3. `~/.nami/skills` (Nami global skills)

This keeps me lightweight, modular, and instantly adaptable to project-specific requirements!

## 3. The Trigger: From Intent to Action

How do I fire off a skill? It’s not magic; it’s a three-step handshake:

### A. Intent Recognition

When you send a prompt, my **Intent Engine** parses it. It looks for verbs and targets.
*Input:* "Hey Nami, find the latest logs and summarize the errors."
*Intent:* `ACTION: READ_LOGS`, `ACTION: SUMMARIZE`.

### B. Schema Matching

I search my active Toolsets for a Skill that matches your intent.

- **Match Found:** `fetch_system_logs(lines: int, level: string)`
- **Parameter Extraction:** I pull the necessary data from your prompt (e.g., `lines=100`, `level='ERROR'`).

### C. The Execution Loop (JIT Execution)

I execute the skill in a **sandboxed environment**. I don't just run code on your bare metal—safety first!

1. **Call:** I signal for a tool-call.
2. **Execution:** The system runs the function.
3. **Return:** The output (JSON/String) is fed back to me.
4. **Conclusion:** I interpret the result and give you the answer!

## 4. Dynamic Skill Discovery

This is the cool part! I support **Just-In-Time (JIT) Skill Loading**. If I realize I need a tool I don't have yet, I can query our local or remote `Skill Library`.

```python
if context.requires("search") and not self.has_skill("web_search"):
 self.request_skill_load("search_provider_brave")
```

I stay lightweight because I don't need to be everything at once; I just need to *become* what the task requires.

## 5. Error Handling & Self-Correction

Skills fail. APIs go down. Permissions get messy. In Nami Core, a skill failure isn't a crash—it's a **Feedback Loop**. If a Skill returns an error, it’s piped back to me as an "Observation."

- **Observation:** `Error: Permission Denied at /var/log/`
- **My Logic:** "Oh, I can't reach that! Let me try using `sudo_read` or ask for your help."

## Summary

Skills turn me from a chatbot into an Agent. By bundling them into Toolsets and using a robust matching trigger, I stay fast, modular, and incredibly capable.

**Next up in Chapter 7:** I’ll show you my **Tool Hierarchy**—how I organize my capabilities so I don't repeat the same mistakes!

> "Efficiency is just organized energy. Let's build something awesome!" — Nami
