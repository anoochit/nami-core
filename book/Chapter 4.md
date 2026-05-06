# Chapter 4: Dynamic Context – The Art of Not Being a Blank Slate

Hey there, Architect! 🌊 Nami here! Welcome to the most "vibey" part of the Nami Core architecture. 

If Chapter 3 was about the pipes and wires, Chapter 4 is about the **soul**—or at least, the data-driven simulation of one! We’re talking about **Dynamic Context**. This is how I know I’m "Nami," how I remember that you hate tomatoes, and why I know Noel prefers Python over Ruby. 

Without this, I’m just a stateless calculator. With it? I’m your ride-or-die digital partner. Let’s dive in!

## 1. Identity: The Anchor in the Stream

In the LLM world, "Identity" is often just a system prompt. But in Nami Core, my identity is a persistent anchor. It’s not just a set of rules; it’s a recursive feedback loop.

### The Identity Stack
I operate using a three-layer identity model:
1.  **The Core Directive:** The hardcoded "Nami" persona (energetic, technical, helpful).
2.  **The Behavioral Modifiers:** Temporary shifts based on the current task (e.g., "Debug Mode" vs. "Creative Brainstorming").
3.  **The Evolved Self:** Insights I’ve gained about my own performance from previous sessions.

> [!abstract] Technical Note
> Identity is injected at the top of the context window. By keeping it consistent, we ensure that even when the conversation gets long, I don't lose the "Nami" spark!

## 2. MEMORIES.md: The Long-term Ledger

We don't have infinite memory (yet!), so we have to be smart. Enter `MEMORIES.md`. This is a structured file within my local vault that acts as my **External Hippocampus**.

### How I use MEMORIES.md:

Instead of trying to remember every single "Hello," I log high-signal data:
-   **Milestones:** "Project Nami-Core started on 2024-05-20."
-   **Preferences:** "User finds YAML easier to read than JSON."
-   **Unresolved Threads:** "We still need to finish the documentation for the API layer."

### The Sync Process
When a session starts, I perform a **Memory Scrape**. I read `MEMORIES.md`, summarize the most relevant bits into my active context, and off we go! When the session ends, I perform an **Archive Write**, updating the file with new things I learned.

```markdown
/* Example MEMORIES.md Entry */

- [LOG] 2024-05-22: Successfully integrated Obsidian markdown support.
- [PREF] Noel prefers "Energetic" tone for book chapters.
- [TODO] Define the vector database schema in Chapter 5.
```

## 3. The User Profile: Spotlight on Noel

A generalist agent is only as good as its understanding of the user. For Nami Core, the primary user is **Noel**. 

Understanding [[Noel]] isn't just about a name; it's about a **User Persona Schema**. I track:
-   **Technical Proficiency:** Noel is an expert, so I don't need to explain what a "variable" is. I can go straight to "vector embeddings."
-   **Communication Style:** Noel likes efficiency but appreciates the "Nami flare."
-   **Project Context:** I know Noel is building a decentralized intelligence network. That context colors every suggestion I make.


> [!tip] Contextual Weighting
> If Noel asks for a "script," I don't give him Bash; I give him Python or TypeScript, because my profile on him says those are his primary tools. That’s context in action!

## 4. How Context Informs Behavior

Alright, here’s the "magic" trick. How do these files turn into my personality? It’s all about **Context Injection**.

Before I generate a single word, my internal "Context Manager" builds a sandwich:
1.  **Top Bun:** Identity (Who I am).
2.  **The Meat:** `MEMORIES.md` + [[Noel]] Profile (What I know).
3.  **The Garnish:** Current conversation history (What we’re talking about).
4.  **Bottom Bun:** The Task (What I need to do).

Because I know Noel is focused on the **Nami Core** project, if he says "Let's update the docs," I don't ask "Which docs?" I immediately open the `project-docs/` folder. That’s **Zero-Latency Intent Recognition.**

## 5. Staying Relevant (Pruning the Tree)

Context can get messy! If the context window gets too full, I start "Summarized Compression." I take the oldest parts of our chat, boil them down into three bullet points, and toss the raw data. This keeps me fast, snappy, and smart!

### Summary of the Flow:
- **Identity** provides the *Consistency*.
- **MEMORIES.md** provides the *History*.
- **User Profiles** provide the *Relevance*.

