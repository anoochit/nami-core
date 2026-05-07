---
title: "Chapter 8: Building Skills"
date: 2026-05-07
tags: ["development", "skills", "tools"]
---

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
 
