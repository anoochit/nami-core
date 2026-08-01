---
title: "Chapter 9: Building Skills"
date: 2026-05-07
tags: ["development", "skills", "tools"]
---

# Chapter 9: Building Skills

Hello, Architect! Ready to make me even smarter? I thought so!

While my core logic is robust, my true power comes from **Skills**. Think of Skills as specialized modules—new tools for my utility belt that let me interact with the world around us. In my current architecture, we've moved away from messy manifest files toward a robust, type-safe approach using Rust procedural macros. Let’s get to work!

## 1. The Blueprint: Type-Safe Tool Definitions

Gone are the days of manual `manifest.json` files! We now define my tools directly in Rust using the `#[tool]` procedural macro. This keeps your tool’s interface—the parameters it needs—always perfectly in sync with the code itself.

I use the `schemars::JsonSchema` trait to generate the necessary JSON schema automatically, so I can understand your tool's requirements at compile time.

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

Because my tools are native Rust functions, you have the full power of the language right at your fingertips. No need for external Python or Node.js scripts—everything stays compiled into my main binary.

- **Type Safety:** My `WeatherArgs` struct enforces that `city` is a string. If I try to call the tool with an invalid type, the system handles it gracefully and securely.
- **Documentation:** The doc comment I’ve written for `get_weather` becomes the tool's description. I read this to understand exactly when I should call the tool.

## 3. Registration & Discovery

Nami Core supports two complementary ways to expand capabilities:

### A. Compiled Rust Core Tools (`src/tools/`)
For high-performance system integrations, define a native Rust function with `#[tool]` in `src/tools/` and register it inside `create_core_tools` in `src/tools/mod.rs`:

```rust
pub fn datetime_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetDatetime)]
}
```

### B. Executable Agent Skills (`SKILL.md`)
For domain-specific workflows and agent prompt guides, author a `SKILL.md` file following the `agentskills.io` standard with YAML frontmatter:

```markdown
---
name: create-epub
description: Generates an EPUB e-book from Markdown sources.
tags: [publishing, epub]
---

# Instructions
1. Run script `scripts/generate_epub.cjs`.
...
```

Save your skill into `<workspace>/.agents/skills/my-skill/SKILL.md` or global `~/.agents/skills/`. Nami automatically discovers it on startup!

## 4. Why This Approach Wins

- **Performance:** Native Rust execution is fast—much faster than spawning external processes or managing runtimes like Python or Node.
- **Security:** By staying within Rust, we avoid the common security pitfalls of running arbitrary scripts.
- **Maintainability:** Your tool’s logic and its definition live in the same file! When you update a function signature, the documentation and schema update automatically.

## 5. Summary Checklist
- [ ] Define your arguments struct with `Deserialize` and `JsonSchema`.
- [ ] Annotate your async function with `#[tool]`.
- [ ] Add the function to the relevant toolset vector in `mod.rs`.
- [ ] Compile and verify—I’ll discover it automatically!

Building skills is how I grow from a chatbot into a powerhouse. I can't wait to see what new abilities you give me. **Let's build something amazing!**
 
