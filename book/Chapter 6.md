# Chapter 6: Skill-Based Execution 🛠️

Alright, let’s get into the gears and grease! Thinking is great—don't get me wrong, I love a good logical loop—but a generalist agent that can’t *do* anything is just a fancy calculator. 

In **Nami Core**, we bridge the gap between "knowing" and "doing" through **Skills**. If the LLM is my brain, Skills are my hands, my eyes, and my multi-tool. This chapter breaks down how we define, bundle, and trigger these capabilities to interact with the real world.

## 1. What is a "Skill"?

A Skill is a discrete, modular function that allows me to interface with an external environment. Whether it's reading a file, pinging an API, or generating an image, every action is encapsulated as a Skill.

In technical terms, a Skill is a **Schema + Logic** pairing:
- **The Schema:** A JSON-based description (often following the OpenAI Tool/Function calling format) that tells me *what* the skill does and *what arguments* it needs.
- **The Logic:** The underlying code (Python, JavaScript, or Bash) that executes the task.

## 2. Bundling: The "Toolset" Concept

We don't just throw raw functions at the wall! To keep my context window clean and my processing efficient, we use **Skill Toolsets**. 

Bundling allows me to load specific "profiles" depending on the task at hand. Why carry a soldering iron to a poetry slam? 

### The Toolset Manifest
Every Toolset contains a `manifest.yaml` that defines its scope.

```yaml
toolset_id: "dev_ops_plus"
version: "1.2.0"
capabilities:
  - file_write
  - git_commit
  - docker_status
  - log_parser
dependencies: ["python-docker-sdk", "gitpython"]
```

By grouping these, the **Nami Core** orchestrator can swap out my active abilities on the fly. If you tell me "Deploy the app," I swap to the `DevOps` bundle. If you say "Write a report," I switch to `OfficeSuite`.

## 3. The Trigger Mechanism: Intent to Action

How do I actually fire off a skill? It’s not magic; it’s a three-step handshake:

### A. Intent Recognition
When a prompt enters my system, the **Intent Engine** parses the request. It looks for verbs and targets. 
*Input:* "Hey Nami, find the latest logs and summarize the errors."
*Intent:* `ACTION: READ_LOGS`, `ACTION: SUMMARIZE`.

### B. Schema Matching
I look through my active Toolsets to find a Skill whose description matches the Intent. 
- **Match Found:** `fetch_system_logs(lines: int, level: string)`
- **Parameter Extraction:** I extract the necessary data from your prompt (e.g., `lines=100`, `level='ERROR'`).

### C. The Execution Loop (JIT Execution)
The Core executes the skill in a **sandboxed environment**. I don't just run code on your bare metal—safety first!

1. **Call:** I emit a tool-call signal.
2. **Execution:** The system runs the function.
3. **Return:** The output (JSON or String) is fed back into my context.
4. **Conclusion:** I interpret the result and give you the answer!

## 4. Dynamic Skill Discovery

This is the cool part! Nami Core supports **Just-In-Time (JIT) Skill Loading**. If I realize I don't have the tool I need, I can query a local or remote `Skill Library`.

```python
if context.requires("search") and not self.has_skill("web_search"):
    self.request_skill_load("search_provider_brave")
```

This makes me lightweight. I don’t need to be everything all at once; I just need to be able to *become* what the task requires.

## 5. Error Handling & Self-Correction

Skills fail. APIs go down. Disk permissions get messy. In Nami Core, a skill failure isn't a crash—it's a **Feedback Loop**. If a Skill returns an error, the output is piped back to me as a "Observation."

- **Observation:** `Error: Permission Denied at /var/log/`
- **Nami's Logic:** "Oh, I can't reach that. Let me try using `sudo_read` or asking the user for elevated access."

## Summary

Skills turn an LLM into an Agent. By bundling them into Toolsets and using a robust schema-matching trigger, Nami Core stays fast, modular, and incredibly capable. 

**Next up in Chapter 7:** We’ll look at **Memory Systems**—how I remember what I did with those skills so I don't repeat the same mistakes!

> "Efficiency is just organized energy. Let's build something awesome!" — Nami