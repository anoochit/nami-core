---
title: "Chapter 5: Long-term Memory"
date: 2026-05-07
tags: ["okf-catalog", "open-knowledge-format", "persistence", "knowledge-management"]
---

# Chapter 5: Long-term Memory – Open Knowledge Format (OKF v0.2)

Alright, Team! System check is green. We’ve talked about how I flow and reason, but now it’s time to talk about **persistence**. If we don't remember what we built yesterday, we're just spinning our wheels in the mud!

In **Nami Core**, memory isn't just a unstructured database entry—it’s a living, machine-readable, and human-friendly **Knowledge Catalog** built on the **Open Knowledge Format (OKF v0.2)** specification.

## 1. The `wiki/` Vault: Our Open Knowledge Neocortex

In my architecture, the `wiki/` directory acts as our **Open Knowledge Catalog**. While session logs tell us what *happened*, the Knowledge Catalog tells us what things *are*, how they *work*, and how much we can *trust* them.

When you or I learn something mission-critical—a tricky API quirk, a deployment workflow, an architecture decision, or an attested computation—it gets promoted to an OKF concept file in the vault.

### Why Open Knowledge Format (OKF v0.2)?
OKF v0.2 bridges the gap between human intuition and agentic verification:
- **First-Class Metadata (`type`):** Every document has a required `type:` field (e.g., `Concept`, `Playbook`, `Metric`, `Attested Computation`, `API Endpoint`).
- **Provenance & Attestation (`sources` & `generated`):** Captures who created the knowledge (`generated: { by: "agent:nami", at: "..." }`), its external sources (`sources:`), and who verified it (`verified:`).
- **Trust Tiers & Freshness:** Automatically derives trust tiers (`unverified`, `machine-confirmed`, `human-reviewed`) and flags expired knowledge via `stale_after: YYYY-MM-DD`.
- **Standard Markdown Links & Wikilinks:** Connects concepts cleanly via standard Markdown paths (`[Customers](/tables/customers.md)`) while retaining backward-compatible `[[wikilinks]]`.
- **Progressive Disclosure (`index.md`):** Uses an `index.md` catalog at the root (carrying `okf_version: "0.2"`) to let humans and agents browse available knowledge before opening detailed files.

## 2. The "Knowledge before Search" Protocol

This is my golden rule: **Check our internal Knowledge Catalog before we hit the open web.**

### Why?
1. **Context is King:** Google knows how *the world* writes code; our OKF Catalog knows how *we* build architecture for *this* specific project.
2. **High Signal & Attestation:** OKF documents carry provenance and trust metadata, eliminating hallucinated or outdated noise.
3. **Prevention of Drift:** If we solve a bug once and record it as an OKF concept, we never have to re-discover that solution again.

**Our Protocol Flow:**
1. **Query:** "How do we rotate the API keys?"
2. **Scan:** Check `wiki/Operations/Security.md` or query concepts by `type: Playbook`.
3. **Execute:** Follow our verified, attested steps.
4. **Fallback:** Only if the Catalog is silent do we go external.
5. **Promote:** Once resolved, **we bring that knowledge home** as an OKF concept document immediately.

## 3. Knowledge Promotion (Our "Learning" Loop)

Long-term memory is only useful if it’s accurate and verifiable. In Nami Core, we utilize **Knowledge Promotion**:

- **Tier 1: Transient (Chat):** Ideas and debugging steps are discussed in the session.
- **Tier 2: Documented (Logs):** Execution events are captured.
- **Tier 3: Permanent (OKF Catalog):** Verified concepts, playbooks, and metrics are saved with OKF v0.2 frontmatter (`status: stable`).

When I’m operating as your agent, I’m constantly scanning for "Knowledge-worthy" moments. If we spend 30 minutes debugging a Docker issue, my final step is drafting `wiki/DevOps/Docker-Network-Fix.md` with full OKF provenance tags.

## 4. Visualizing the Knowledge Topology

Memory isn't just storage; it's about **topology**. My `get_wiki_graph` tool allows us to visualize the connections between our concepts.

When concepts are linked via standard Markdown links or `[[wikilinks]]`, I can generate a graph map of our project's knowledge. This helps us find isolated concepts and central architecture hubs, ensuring our memory remains a cohesive system.

## 5. Structuring the Catalog for Retrieval

To keep our memory efficient, I organize concepts logically:
- `index.md`: Root index with `okf_version: "0.2"` enumerating concepts for progressive disclosure.
- `/playbooks`: Operational step-by-step guides (`type: Playbook`).
- `/specs`: Technical architecture & schema concepts (`type: Concept` or `type: API Endpoint`).
- `/computations`: Attested computations (`type: Attested Computation`).
- `log.md`: Chronological history of updates.

## 6. Technical Implementation: RAG-Ready & Attestable

By keeping our knowledge in clean, OKF-compliant Markdown files, we are "RAG-Ready" and "Attestation-Ready." I can parse these concepts, inspect their `verified` status, vectorize their content, and inject verified facts into my context window with surgical precision.

### Final Logic Check
The `wiki/` directory isn't a random graveyard for notes; it’s our **Open Knowledge Catalog**. By prioritizing verified OKF knowledge, we ensure that every hour we spend working makes the system smarter, safer, and more autonomous.

**Let’s get to work. Documented, attested knowledge is momentum!**
