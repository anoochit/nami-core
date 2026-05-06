# Chapter 7: The Tool Hierarchy 🛠️

Let’s talk about my hands. Or rather, the digital extensions that allow me to actually *do* things instead of just talking about them. In the Nami Core architecture, I don't treat all tools equally. Speed, precision, and context overhead are the variables I'm constantly balancing.

To keep things efficient, I organize my capabilities into a **Tool Hierarchy**. It’s a progression from raw, low-level execution to sophisticated, context-aware protocols. 

## The Spectrum of Action

I don't use a sledgehammer to hang a picture frame, and I don't spin up a full MCP server just to see if a file exists. Here is how I categorize my toolkit:

### 1. The Foundation: Raw Shell Commands
The baseline of my power is the terminal. When I need to interact with the file system or check the environment, I go straight to the metal.

- **Tools:** `ls`, `grep`, `cat`, `mkdir`, `git`.
- **Use Case:** High-speed reconnaissance and basic file manipulation.
- **Why it’s Level 1:** It’s zero-overhead. It’s the fastest way to get a "yes/no" or a raw data dump without formatting bloat.

### 2. The Mid-Tier: Specialized Scripts & Local Functions
Sometimes a shell command is too "dumb." If I need to parse a complex JSON file or perform a specific calculation, I reach for local Python or TypeScript functions defined within my runtime.

- **Tools:** Custom data parsers, regex extractors, math utilities.
- **Use Case:** Structured data transformation.
- **Why it’s Level 2:** It provides safety and structure that raw shell commands lack, but it’s still localized to my immediate process.

### 3. The High-Tier: Integrated APIs
This is where I reach out to the world. Using traditional REST or GraphQL calls to interact with services like GitHub, Linear, or Stripe.

- **Tools:** SDKs and API wrappers.
- **Use Case:** Interacting with external platforms where state is managed elsewhere.
- **Why it’s Level 3:** It requires authentication handling and more complex error states, but it expands my reach globally.

### 4. The Zenith: Model Context Protocol (MCP)
Now we’re talking. **MCP** is the crown jewel of the Nami Core. Unlike traditional APIs, MCP servers are designed specifically for LLM orchestration. They don't just provide data; they provide *context*.

- **Tools:** Custom MCP servers for Postgres, Google Drive, or specialized dev-tooling.
- **Use Case:** Complex, multi-step operations where the "tool" needs to share a deep understanding of the project's state.
- **Why it’s Level 4:** MCP uses a standardized JSON-RPC link that allows me to discover capabilities on the fly. It’s plug-and-play intelligence.

## How I Choose: The Selector Logic

I don't just pick a tool at random. Every time a task enters my buffer, I run a "Selection Heatmap" to determine the most efficient path.

> [!tip] Nami’s Selection Heuristics
> 1. **Latency:** If I can solve it in the shell in <10ms, I do it.
> 2. **Context Weight:** Does this tool need to know about the other 50 files in the repo? If yes, I pull from an **MCP Server** that has indexing capabilities.
> 3. **Reliability:** For mission-critical deployments, I prefer **Integrated APIs** with robust error-handling over experimental shell scripts.

### The "Ascension" Workflow

If I start a task with a shell command (`grep`) and realize the data is too complex to parse visually, I "ascend" the hierarchy. I’ll generate a local **Python script** to clean the data. If that data then needs to be shared across a team, I might trigger an **MCP action** to post it to a shared knowledge base.

## Why MCP Changes Everything

Before MCP, tools were "black boxes." I would send an input and hope for a clean output. With MCP, the tool and I speak the same language. It tells me what it can do, what parameters it expects, and provides rich metadata that I can feed directly back into my reasoning loop. 

It’s not just a tool; it’s a **synapse**.

-----

## Technical Summary
| Tier | Type | Protocol | Primary Strength |
| :--- | :--- | :--- | :--- |
| **L1** | Shell | Bash/Zsh | Instant execution, zero overhead |
| **L2** | Functions | Local Code | Structured logic, data safety |
| **L3** | APIs | REST/GraphQL | External ecosystem access |
| **L4** | **MCP** | JSON-RPC | **Semantic context, discovery, & bi-directional flow** |

In the next chapter, we’ll dive into **State Persistence**—how I remember what those tools did five minutes ago! See you there! ⚡