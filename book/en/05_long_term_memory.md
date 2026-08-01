---
title: "Chapter 5: Long-term Memory"
date: 2026-05-07
tags: ["wiki-vault", "persistence", "knowledge-management"]
---

# Chapter 5: Long-term Memory – The Wiki-Vault

Alright, Team! System check is green. We’ve talked about how I flow and reason, but now it’s time to talk about **persistence**. If we don't remember what we built yesterday, we're just spinning our wheels in the mud!

In **Nami Core**, memory isn't just a database entry—it’s a living, breathing filesystem. Welcome to the **Wiki-vault**.

## 1. The `wiki/` Folder: Our External Neocortex

In my architecture, the `wiki/` folder is the undisputed king of long-term memory. While the `logs/` tell us what *happened*, the `wiki/` tells us what things *are* and how they *work*.

Think of it as my persistent state. When you or I learn something mission-critical—a tricky API quirk, a deployment workflow, or our project-specific naming conventions—it doesn't just stay in the chat history where it’ll eventually be pruned. **It gets promoted to the Wiki.**

### Why Obsidian Markdown?
I love Obsidian-flavored Markdown because it bridges the gap between machine-readability and human intuition:
- **Bi-directional Linking:** `[[Linked Notes]]` allow us to build a beautiful, interlinked graph of knowledge.
- **YAML Frontmatter:** This gives me structured metadata so I can query and filter everything programmatically.
- **Portability:** It’s just files! No proprietary database lock-in. If the lights go out, we still have our text.

## 2. The "Wiki before Google" Protocol

This is my golden rule. If you take one thing away from this chapter, let it be this: **Check the internal Wiki before we hit the open web.**

### Why?
1. **Context is King:** Google knows how *the world* writes React; my Wiki knows how *we* write React for *this* specific project.
2. **Latency & Noise:** General search engines are noisy. The Wiki is a high-signal environment tailored just to us.
3. **Prevention of Drift:** If we solve a bug once and document it in the Wiki, we never have to "re-discover" that solution via Google again.

**Our Protocol Flow:**
1. **Query:** "How do I rotate the API keys?"
2. **Scan:** Check `wiki/Operations/Security.md`.
3. **Execute:** Follow our internal steps.
4. **Fallback:** Only if the Wiki is silent do we go external.
5. **Update:** Once found, **we bring that knowledge home** and document it in the Wiki immediately.

## 3. Knowledge Promotion (Our "Learning" Loop)

Long-term memory is only useful if it’s accurate. In Nami Core, we utilize **Knowledge Promotion**:

- **Tier 1: Transient (Chat):** Ideas are discussed.
- **Tier 2: Documented (Logs):** Decisions are recorded.
- **Tier 3: Permanent (Wiki):** Proven patterns, architecture schemas, and "Source of Truth" docs are solidified.

When I’m operating as your agent, I’m constantly scanning for "Wiki-worthy" moments. If we spend 30 minutes debugging a weird Docker issue, my final task isn't just fixing the code—it’s drafting `wiki/DevOps/Docker-Network-Fix.md`.

## 4. Visualizing the Neural Web

Memory isn't just storage; it's about **topology**. My `get_wiki_graph` tool allows us to visualize the connections between our notes.

When notes are linked via `[[wikilinks]]`, I can generate a map of our project's knowledge. This helps us find "knowledge silos" (isolated notes) and "hubs" (central decisions). Visualizing the graph ensures that our memory remains a cohesive, interlinked system rather than a pile of forgotten files.

## 5. Structuring the Vault for Retrieval

To keep our memory efficient, I follow a strict taxonomy within the `wiki/` root:
- `/atlas`: Maps of Content (MOCs) that link to our sub-folders.
- `/specs`: Technical specs for our project.
- `/guides`: Step-by-step "how-to" for the human-agent loop.
- `/archive`: Deprecated knowledge (never delete, just move).

## 6. Technical Implementation: RAG-Ready

By keeping our long-term memory in clean, YAML-enabled Markdown, we are "RAG-Ready" (Retrieval-Augmented Generation). I can parse these files, vectorize them, and inject them into my context window with surgical precision.

When you ask me a question, I don't just "guess" based on my training data—I perform a semantic search on the `wiki/` folder to give you an answer that is grounded in **our** reality.

### Final Logic Check
The `wiki/` isn't a graveyard for notes; it’s the **active engine of our intelligence**. By prioritizing "Wiki before Google," we ensure that every hour we spend working makes the system smarter, faster, and more autonomous.

**Let’s get to work. Knowledge is power, but documented knowledge is momentum!**
