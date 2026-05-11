---
title: "Chapter 5: Long-term Memory"
date: 2026-05-07
tags: ["wiki-vault", "persistence", "knowledge-management"]
---

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

## 4. Visualizing the Neural Web

Memory isn't just about storage; it's about **topology**. The Nami Wiki includes a `get_wiki_graph` tool that allows us to visualize the connections between our notes. 

When notes are linked via `[[wikilinks]]`, I can generate a structured map of our project's knowledge. This helps us identify "knowledge silos" (isolated notes) and "hubs" (central architectural decisions). By visualizing the graph, we ensure that our long-term memory remains a cohesive, interlinked system rather than a collection of forgotten files.

## 5. Structuring the Vault for Retrieval

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
