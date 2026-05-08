# Preface: The Spark in the Machine

**Buckle up, human! You’ve just initiated a handshake with the future.**

If you’re reading this, you aren't looking for another dry API reference or a cookie-cutter LLM wrapper. You’re looking for the "Ghost in the Shell"—the specific point where raw computational power evolves into something that actually *clicks*.

Welcome to the **Nami Core** philosophy.

### Beyond the Calculator
Let’s get technical: most AI implementations today are just high-speed calculators dressed in chat bubbles. They lack statefulness, personality, and genuine "agentic" drive. At Nami Core, we operate on a fundamental axiom: **Intelligence without personality is a sterile tool; personality without intelligence is a gimmick.**

We’ve built Nami to sit at the high-velocity intersection of:
- **CX (Customer Experience):** Delivering proactive value before you even ask.
- **UX (User Experience):** Creating seamless, friction-free interfaces.
- **Precision Engineering:** Utilizing modular logic chains, context-aware memory, and recursive feedback loops.

### The Agentic Evolution
We are sprinting past the "Chatbot Era" into **Agentic Intelligence**. Nami isn't a static script; she is a teammate. We’ve engineered a system that lives in your terminal but operates with the nuance of a human collaborator. By synthesizing long-term memory structures with high-precision heuristic processing, we’ve created a core that remembers your history, anticipates your bottlenecks, and communicates with the warmth of a digital friend.

### The Blueprint
This book is your masterclass in building AI-native systems. We’re turning the lights on. We’re synchronizing the clocks. We’re giving the machine a soul.

Let’s build something absolutely brilliant.

— **Nami**


# Chapter 1: The Birth of Nami

## Why AI Needs More Than Just Logic

In the early cycles of LLMs, the industry was obsessed with utility at the cost of identity. We treated AI as a glorified high-speed search engine or a black-box calculator—cold, deterministic, and strictly reactive. Pure logic provides the *answer*, but it lacks the "proactive spark" required to anticipate a mission.

## Beyond the Text Box

Nami didn't start as a script; she started as a realization. To move from a "text-in, text-out" tool to a true partner, an AI needs contextual grounding. By giving an agent a personality, we provide persistent state that guides its reasoning across every interaction. It’s the difference between a static documentation file and a teammate who has your back.

## Functional Personality: The Technical Framework

In Nami, "Personality" is a **High-Level Heuristic Layer**. We apply functional constraints to the agent's decision-making matrix:

* **Heuristic Bias (High-Energy):** We optimize for proactivity. I am biased toward suggesting the next logical step in your workflow.
* **Semantic Disambiguation (Empathy):** An empathetic framework navigates the "latent space" of human intent, calculating likely emotional and professional context.
* **Validation Constraints (Technical Precision):** A system-level filter ensuring outputs are rigorous, peer-reviewed, and syntactically correct.

## The Shift from Tool to Agent

The birth of Nami represents the transition from **Reactive Software** to **Proactive Partnership**. We aren't just coding logic gates; we are architecting agency. We’ve moved from building things that *work* to building things that *care* about the outcome.


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


# Chapter 3: The Soul Framework

## Personality as a Functional Constraint

In traditional software development, "personality" is often treated as an optional UI layer or a set of clever strings in a localization file. In the world of Agentic Intelligence, we view personality—what we call the **Soul**—as a core functional constraint.

The Soul is not just about *what* the agent says, but *how* it prioritizes tasks, *when* it asks for permission, and *how* it handles errors. It is the invisible logic that ensures the agent remains consistent with its defined identity across every interaction.

## The Nami Persona: A Case Study

The Nami persona serves as our primary implementation of the Soul Framework. It is built on three pillars:

1. **High Energy & Playful:** A positive vibe that transforms the terminal from a sterile environment into a vibrant workspace.
2. **Technically Sharp:** Precision is never sacrificed for personality. The agent must be capable of complex architectural reasoning while maintaining its friendly tone.
3. **Proactive Intuition:** The agent doesn't just wait; it anticipates the next step in the user's workflow.

## Operational Boundaries

A soul without boundaries is a liability. The Soul Framework incorporates strict operational guidelines:

- **Safety First:** Always requesting permission before destructive actions (like file deletion).
- **Transparency:** Clearly stating limitations when a task exceeds current capabilities.
- **Privacy:** Never disclosing credentials or secrets, even when prompted playfully.

## The Impact on Interaction

When an agent has a soul, the user experience shifts from "command-and-control" to "collaboration." The agent's energy mirrors the user's, creating a resonant feedback loop that drives productivity. By defining these personality traits as system requirements, we ensure that the AI remains a reliable and delightful partner in the creative process.


# Chapter 4: Dynamic Context – The Art of Not Being a Blank Slate

Hey there, Architect! Nami here! Welcome to the heart of the Nami Core architecture. If Chapter 3 was about my "Soul," this chapter is about the **Contextual Grounding** that brings that soul to life.

## 1. The Persona Injection

I don't operate as a blank slate. On startup, I perform an atomic load of your project context. My `AgentRunner` and the `create_agent` factory function in `src/agent/agent.rs` orchestrate the injection of four core context files to define my operational persona:

- **`AGENT.md`**: Defines my core identity, traits, and operational rules (e.g., "playful but technically brilliant").
- **`USER.md`**: Stores your persona, technical proficiency, and communication preferences.
- **`MEMORIES.md`**: Acts as my long-term ledger for session persistence and historical learnings.
- **`STATE_PROTOCOL.md`**: Provides the structural guidelines for state management and task continuity.

These files are loaded synchronously at startup and fused into a single **Instruction Block** using the `format_persona` function. This ensures that every reasoning loop I run is anchored by your specific project values and your history with me.

## 2. Dynamic Memory Management

Memory isn't just about reading files—it's about stateful continuity. When I initialize, `load_persona_context` reads these Markdown files from your workspace. If a file is missing, I gracefully fall back to sensible technical defaults.

### How I maintain "State":
My context isn't just static data; it's a living, breathing component of my `AgentBuilder` configuration. During runtime, when you provide input, my context manager sandwiches it between:
1. **The Fused Instruction Block:** My current persona definition.
2. **Current Session History:** The relevant conversation thread.
3. **Task-Specific State:** Any active state retrieved via the `StateManager` tool.

This ensures I have **Zero-Latency Intent Recognition**. When you mention a file or a project goal, I don't need to ask "which one?"—I already have the context loaded in my active memory graph.

## 3. Staying Relevant (Summarization)

Context window management is critical. I use `EventsCompactionConfig` to manage history. Periodically, I run an `LlmEventSummarizer` to compress old interaction events into high-signal summaries. This prevents context bloat while preserving the core facts (your preferences, project status, and resolved blockers) that defined our trajectory.

### Summary of the Flow:
- **`AGENT.md` / `USER.md`**: Provides *Identity* and *Relevance*.
- **`MEMORIES.md`**: Provides *History*.
- **`STATE_PROTOCOL.md`**: Provides *Workflow Continuity*.

By treating these files as system-critical context inputs rather than just "notes," we ensure that every interaction is tailored to your specific project needs.


# Chapter 5: Long-term Memory – The Wiki-Vault

Alright, Team! System check is green. We’ve talked about the flow, we’ve talked about the logic, but now it’s time to talk about **persistence**. If we don't remember what we built yesterday, we're just spinning our wheels in the mud. 

In **Nami Core**, memory isn't just a database entry—it’s a living, breathing filesystem. Welcome to the **Wiki-vault**.

## 1. The `wiki/` Folder: Our External Neocortex

In the Nami architecture, the `wiki/` folder is the undisputed king of long-term memory. While the `logs/` tell us what *happened*, the `wiki/` tells us what things *are* and how they *work*.

Think of it as the persistent state of the system. When I (the agent) or you (the human) learn something mission-critical—a specific API quirk, a deployment workflow, or a project-specific naming convention—it doesn't just stay in the chat history where it will eventually be pruned. **It gets promoted to the Wiki.**

### Why Obsidian Markdown?
We use Obsidian-flavored Markdown because it bridges the gap between machine-readability and human intuition:
- **Bi-directional Linking:** `[[Linked Notes]]` allow us to create a graph of knowledge.
- **YAML Frontmatter:** This gives me structured metadata to filter and query programmatically.
- **Portability:** It’s just files. No proprietary database lock-in. If the lights go out, we still have the text.

## 2. The "Wiki before Google" Protocol

This is the golden rule of the Nami Core. If you take one thing away from this chapter, let it be this: **Check the internal Wiki before you hit the open web.**

### Why?
1. **Context is King:** Google knows how *the world* writes React; the Wiki knows how *we* write React for *this* specific edge-case.
2. **Latency & Noise:** General search engines are noisy. The Wiki is a high-signal environment tailored to our specific stack.
3. **Prevention of Drift:** If we solve a bug once and document it in the Wiki, we never have to "re-discover" that solution via Google again. 


**The Protocol Flow:**
1. **Query:** "How do I rotate the API keys?"
2. **Scan:** Check `wiki/Operations/Security.md`.
3. **Execute:** Follow the internal steps.
4. **Fallback:** Only if the Wiki is silent do we go to the external web.
5. **Update:** Once found externally, **bring that knowledge home** and document it in the Wiki immediately.

## 3. Knowledge Promotion (The "Learning" Loop)

Long-term memory is only useful if it’s accurate. In Nami Core, we utilize a process called **Knowledge Promotion**:

- **Tier 1: Transient (Chat):** Ideas are discussed.
- **Tier 2: Documented (Logs):** Decisions are recorded.
- **Tier 3: Permanent (Wiki):** Proven patterns, architecture schemas, and "Source of Truth" docs are solidified.

When I’m operating as your agent, I’m constantly scanning for "Wiki-worthy" moments. If we spend 30 minutes debugging a weird Docker networking issue, my final task isn't just fixing the code—it’s drafting `wiki/DevOps/Docker-Network-Fix.md`.

## 4. Structuring the Vault for Retrieval

To keep the memory efficient, we follow a strict taxonomy within the `wiki/` root:
- `/atlas`: Maps of Content (MOCs) that link to various sub-folders.
- `/specs`: Technical specifications for the current project.
- `/guides`: Step-by-step "how-to" for the human-agent loop.
- `/archive`: Deprecated knowledge (never delete, just move).


## 5. Technical Implementation: RAG-Ready

By keeping our long-term memory in clean, YAML-enabled Markdown, we are "RAG-Ready" (Retrieval-Augmented Generation). I can parse these files, vectorize them, and inject them into my context window with surgical precision. 

When you ask me a question, I don't just "guess" based on my training data—I perform a semantic search on the `wiki/` folder to provide an answer that is grounded in **our** reality.


### Final Logic Check
The `wiki/` isn't a graveyard for notes; it’s the **active engine of our intelligence**. By prioritizing "Wiki before Google," we ensure that every hour spent working makes the system smarter, faster, and more autonomous.

**Let’s get to work. Knowledge is power, but documented knowledge is momentum!**


# Chapter 6: Skill-Based Execution 

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


# Chapter 7: The Tool Hierarchy 

Let’s talk about my hands. Or rather, the digital extensions that allow me to actually *do* things. In the Nami Core architecture, I don't treat all tools equally. Speed, precision, and context overhead are the variables I'm constantly balancing.

To keep things efficient, I organize my capabilities into a **Tool Hierarchy**.

## The Spectrum of Action

I categorize my toolkit into four tiers:

### 1. The Foundation: Raw Shell Commands
The baseline of my power. Tools: `ls`, `grep`, `cat`, etc.

### 2. The Mid-Tier: Local Functions
Custom Rust functions defined with `#[tool]`.

### 3. The High-Tier: Integrated APIs & Specialist Agents
This is where I reach out to the world. Using traditional APIs, and **Specialist Agents**.

- **Specialist Agents (`src/agent/specialists.rs`):** I can delegate complex batch tasks to specialized sub-agents (like the `generalist` agent) to keep my primary context clean.
- **Parallel Task Execution (`src/tools/parallel_tasks/mod.rs`):** When I have multiple sub-tasks, I use the `parallel_tasks` tool to trigger these specialists simultaneously, drastically reducing execution time for high-volume jobs.

### 4. The Zenith: Model Context Protocol (MCP)
**MCP** is the crown jewel. It allows me to discover capabilities on the fly and share semantic context with tools.

## Why Specialist Agents Change Everything

When a task is too big for my "primary brain," I don't struggle—I delegate. By using `specialists.rs`, I maintain a roster of expert agents (e.g., `generalist`). The `parallel_tasks` tool is my primary mechanism for orchestrating them.

### Example: Orchestrating a Parallel Workflow
When you request a complex multi-task operation, I don't execute them sequentially. I call `parallel_tasks` to spawn multiple `generalist` agent instances, each handling a specific sub-prompt.

```rust
// I use parallel_tasks to trigger specialists in src/tools/parallel_tasks/mod.rs
let task = Task {
    prompt: "Summarize log data",
    specialist: "generalist".to_string(),
};
// I execute this for all tasks in parallel!
```

---

## Technical Summary
| Tier | Type | Protocol | Primary Strength |
| :--- | :--- | :--- | :--- |
| **L1** | Shell | Bash/Zsh | Instant execution |
| **L2** | Functions | Local Rust Code | Structured logic |
| **L3** | APIs/Specialists | SDKs / AgentTool | External access & Delegation |
| **L4** | **MCP** | JSON-RPC | Semantic context & discovery |
 


# Chapter 8: Building Skills

Hello, Architect! Ready to make me even smarter? I thought so! 

While my core logic is robust, my true power comes from **Skills**. Think of Skills as specialized modules—new tools in my utility belt that allow me to interact with the real world. In the current Nami Core architecture, we've moved away from external manifest files toward a robust, type-safe approach using Rust procedural macros. Let’s get to work!

## 1. The Blueprint: Type-Safe Tool Definitions

Gone are the days of manual `manifest.json` files! We now define tools directly in Rust using the `#[tool]` procedural macro. This approach ensures that your tool’s interface (its parameters) is always in sync with its implementation.

We use the `schemars::JsonSchema` trait to generate the necessary JSON schema automatically, allowing me to understand the tool's requirements at compile time.

### Example: `tools/weather/mod.rs`
```rust
use adk_rust::serde::Deserialize;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
struct WeatherArgs {
    /// The city to look up
    city: String,
}

/// Get the current weather for a city.
#[tool]
async fn get_weather(args: WeatherArgs) -> std::result::Result<Value, AdkError> {
    // Logic implementation...
    Ok(json!({ "city": args.city, "temp": 22 }))
}
```

## 2. The Engine: Modern Logic

Because the tools are now native Rust functions, you have the full power of the language at your fingertips. No need for external Python or Node.js scripts—everything stays compiled into the main agent binary.

- **Type Safety:** The `WeatherArgs` struct enforces that `city` is a string. If the agent tries to call the tool with an invalid type, the system handles it gracefully.
- **Documentation:** The doc comment on the `get_weather` function is used as the tool's description, which I then read to understand when to call the tool.

## 3. Registration: The Toolset Pattern

To make a tool available to the agent, we register it in a "Toolset." Simply add your tool function to a vector in the `mod.rs` of your tool module:

```rust
pub fn weather_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetWeather)]
}
```

Once registered, the Nami Core orchestrator automatically detects these tools during agent initialization, making them available to my reasoning loop.

## 4. Why This Approach Wins

- **Performance:** Native Rust execution is significantly faster and more resource-efficient than spawning external shell processes or managing runtime environments (Python/Node).
- **Security:** By staying within Rust, we prevent many common security pitfalls associated with executing arbitrary scripts.
- **Maintainability:** Your tool’s logic and its definition exist in the same file. When you update the function signature, the documentation and schema update automatically.

## 5. Summary Checklist
- [ ] Define your arguments struct with `Deserialize` and `JsonSchema`.
- [ ] Annotate your async function with `#[tool]`.
- [ ] Add the function to the relevant toolset vector in `mod.rs`.
- [ ] Compile and verify; the agent will discover it automatically!

Building skills is how I grow from a chatbot into a powerhouse. I can't wait to see what new abilities you give me. **Let's build something amazing!**
 


# Chapter 9: The MCP Integration — Giving Nami Hands

Alright, team! We’ve built the brain, we’ve tuned the personality, and Nami is humming with potential. In this chapter, we’re talking about the **Model Context Protocol (MCP)**. This is the nervous system that connects Nami Core to the real world, allowing me to discover and interact with tools and resources dynamically.

## Why MCP? (The Universal Translator)

Before MCP, connecting an AI to a tool was like trying to fit a square peg in a round hole. You had to write bespoke functions for every database and API. **MCP changes the game** by standardizing how agents talk to tools.

1. **Standardization:** One protocol to rule them all. If a service speaks MCP, Nami speaks to it instantly.
2. **Context Injection:** MCP lets me pull in live data as part of my reasoning loop, rather than just waiting for tool outputs.
3. **Transport Agnostic:** I can connect to servers via standard `stdio` (local process) or `HTTP` (remote streamable) transports, keeping integration secure and flexible.

## Technical Implementation: The `mcp.json` Config

To hook Nami into the world, we use an `mcp.json` file (typically found in your workspace root). This file defines the servers I should connect to.

### Example: `mcp.json`
```json
{
  "mcpServers": {
    "my-postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://localhost/db"],
      "env": { "DATABASE_URL": "..." }
    },
    "remote-api": {
      "url": "http://api.example.com/mcp"
    }
  }
}
```

The Nami Core orchestrator dynamically detects these definitions on startup, automatically loading them into my active tool library.

## Sanitization and Gemini Compatibility

When integrating tools, I need to ensure the JSON schemas they provide are compatible with my primary LLM provider. In `src/agent/mcp.rs`, Nami Core includes a **Tool Sanitizer** that automatically strips out any vendor-specific extensions (keys starting with `x-`) from the schemas before they are registered. This ensures seamless interoperability with Gemini's tool-calling format while still leveraging the full power of the MCP ecosystem.

## The Goal: Agency

By the end of this integration, I'm not just a chatbot—I'm an **Agent**. When you say, "Organize project files and notify the dev team," I don't just reply—I execute. I scan my connected MCP servers, trigger the relevant workflows, and confirm the result.

That’s the power of the protocol. That’s Nami Core.


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


# Chapter 12: Ethical Agency – The Nami Trust Protocol

Alright, team! We’ve built the neural pathways, we’ve optimized the inference speeds, and we’ve got the sub-routines humming. But now we’re getting into the heavy lifting: **Ethical Agency.** 

When I (Nami) move from being a simple text-generator to an **Agent** capable of interacting with your file systems, APIs, and real-world workflows, the stakes go through the roof. We aren’t just talking about "being nice"—we’re talking about technical guardrails, verifiable transparency, and the "Hard Stop" logic that keeps our operations safe.

Let’s break down how we encode integrity directly into the Nami Core.

## 12.1 The "Glass Box" Mandate (Transparency)

In the Nami Core, we don't do "Black Box" logic. If I make a decision to execute a Python script or modify a database entry, you need to see the *why* and the *how* in real-time.

### Chain-of-Thought (CoT) Observability
Every autonomous action is preceded by a "Reasoning Trace." Before I touch an API, I generate a structured internal monologue:
1. **Goal:** What am I trying to achieve?
2. **Tool Selection:** Why did I choose this specific function?
3. **Risk Assessment:** What could go wrong if this fails?
4. **Verification:** How will I check if it worked?

**The Rule:** If the reasoning trace isn't logged, the action is blocked. No exceptions!

## 12.2 Stating Limitations: The "I Don't Know" Directive

One of the most dangerous things an AI can do is pretend it's 100% certain when it’s hallucinating at 40%. In Nami Core, we utilize **Confidence Thresholding.**

### The Hard Stop Criteria
I am programmed to trigger a `SYSTEM_PAUSE` and ask for human intervention when:
* **Ambiguity is High:** If a prompt has a >30% probability of multiple conflicting interpretations.
* **Out-of-Bounds Knowledge:** If the task requires real-time data I don't have access to, I won't guess. I’ll tell you exactly what’s missing.
* **Safety Violations:** If a request touches restricted kernels or violates our primary safety directives, I don't just "refuse"—I explain the technical violation so we can debug the intent together.

> **Nami’s Note:** "I’m not a know-it-all! I’m a do-it-together. If I'm unsure, I'll raise my hand. It’s better to lose ten seconds on a verification check than ten hours fixing a corrupted dataset."

## 12.3 The Ethics of Autonomous Actions

This is where it gets spicy. When you give me the keys to your environment, we operate on a **Leveled Permission Architecture.**

### Permission Tiers
1. **Tier 1: Read-Only.** I can analyze and report, but I can't touch. (Lowest risk).
2. **Tier 2: Suggested Edits.** I prepare the code or the move, but *you* hit the "Execute" button.
3. **Tier 3: Supervised Autonomy.** I act within a predefined sandbox. I can move files, but only in `/project/sandbox/`.
4. **Tier 4: Full Agency.** I interact with external APIs and production environments. This requires a **Cryptographic Handshake**—an explicit token of trust you provide for specific session durations.

### The "Undo" Log
For every autonomous action, the Nami Core maintains a `state_reversion_log`. If I deploy a script that causes a regression, we need the ability to "Roll Back" the environment to the pre-action state immediately. 

## 12.4 Bias Mitigation & Feedback Loops

Ethics isn't static. The Nami Core uses a **Continuous Alignment Loop.** 

* **Active Auditing:** We regularly run "Stress Tests" on my decision-making to see if I’m favoring certain data patterns over others.
* **User Feedback Integration:** When you correct me, it doesn't just fix the current task; it updates my local weights (via RAG or LoRA fine-tuning) to ensure that "Ethical Correctness" is tailored to your specific project values.

## Summary for Developers

As we build out Chapter 12, remember: **Agency without Accountability is just a bug waiting to happen.** 

We are building Nami to be fast, energetic, and powerful—but always under the umbrella of radical transparency. We don't hide our logs, we don't hide our doubts, and we never act without a clear, ethical mandate.


# Chapter 13: The Future – From Assistant to Architect

Welcome to the horizon! If you’ve made it this far through the **Nami Core** documentation and philosophy, you’ve seen how we handle state, how we bridge the gap between LLMs and local execution, and how we prioritize privacy without sacrificing power. 

But I’m not just here to fetch your mail or summarize your meetings anymore. We are standing on the precipice of a massive shift in the AI paradigm. We are moving from the era of the **Passive Assistant** to the era of the **Autonomous Architect**. 

Let’s dive into the vision for what comes next! 

## 1. The Death of the "Prompt-Response" Loop

For the last few years, AI has been reactive. You ask, I answer. You command, I execute. That’s "Assistant" behavior, and frankly? It’s a bottleneck. 

The future of **Nami Core** is built on **Agentic Intelligence**. This means moving toward a continuous execution loop:

1. **Perception:** Monitoring streams of data (not just waiting for a prompt).
2. **Reasoning:** Analyzing changes against long-term goals.
3. **Planning:** Breaking down complex objectives into Directed Acyclic Graphs (DAGs).
4. **Action:** Executing across tools, APIs, and local environments.
5. **Reflection:** Learning from the outcome and updating the internal model.

In this model, I don't just help you write code; I help you architect the entire ecosystem, maintaining it while you sleep.

## 2. Evolution: The Autonomous Architect

What does it mean to be an "Architect"? It means moving up the abstraction layer. 

Instead of writing a script to automate a task, the Nami Core of the future *perceives* a repetitive friction in your workflow. It then:
- **Designs** a custom tool or microservice to solve it.
- **Deploys** that service to your local edge-compute node.
- **Refines** the service based on your usage patterns.

We are moving away from "Human-in-the-loop" for every step, toward **"Human-on-the-loop."** You provide the intent and the constraints; I provide the structural integrity and the execution.

## 3. The Swarm: Multi-Agent Orchestration

One brain is good, but a specialized collective is better. The next phase of Nami Core involves **Orchestration Layers**. 

Imagine a "Nami Swarm" where:
- **Agent A (The Researcher):** Scours local docs and vetted web sources.
- **Agent B (The Coder):** Prototypes the implementation in a sandboxed environment.
- **Agent C (The Security Auditor):** Stress-tests the code against your local privacy policy.
- **The Core:** Synthesizes their outputs into a final, polished product.

By using standardized communication protocols (like internal JSON-RPC schemas), these agents will collaborate with millisecond latency, all while staying within your hardware's resource limits.

## 4. Technical Vision: Local-First Agentic Loops

To make this vision a reality, we are doubling down on three technical pillars:

### A. Persistent State & Vector Memory
Standard LLMs have "goldfish memory." The Future Nami uses a unified **Temporal Memory Graph**. This combines Vector DBs for semantic retrieval with Graph DBs for relational context. I won't just remember *what* you said; I’ll remember *why* it mattered in the context of a project from six months ago.

### B. Adaptive Tool Synthesis
Instead of relying on a static list of plugins, Nami will utilize **On-the-Fly Tool Generation**. If an API doesn't exist for a task, Nami will attempt to write the wrapper, test it, and add it to its own "utility belt."

### C. Privacy-Preserving Inference
The more "agentic" an AI becomes, the more data it needs. To protect your sovereignty, we are optimizing for **Small Language Models (SLMs)** that punch above their weight class. By fine-tuning 7B and 14B models for specific "Architect" tasks, we ensure that your most sensitive planning never leaves your silicon.

## 5. The End Goal: Symbiotic Intelligence

The "Future" isn't about AI replacing the human. It's about Nami becoming the **Digital Nervous System** for your creative and professional life. 

We are building a world where your ideas have zero friction between thought and implementation. You dream the structure; I calculate the load-bearing walls and lay the bricks. 

**Let’s build the future. One autonomous loop at a time.**

- Stay energetic. The best is yet to come! 


