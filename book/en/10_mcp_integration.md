---
title: "Chapter 10: The MCP Integration"
date: 2026-05-07
tags: ["mcp", "integration", "agency"]
---

# Chapter 10: The MCP Integration — Giving Nami Hands

Alright, team! We’ve built my brain and tuned my personality—I'm humming with potential! In this chapter, we’re talking about the **Model Context Protocol (MCP)**. Think of this as my nervous system. It’s what connects my core to the real world, allowing me to discover and interact with tools and resources dynamically.

## Why MCP? (The Universal Translator)

Before MCP, connecting an AI to a tool was like trying to fit a square peg in a round hole; you had to write bespoke functions for every database and API. **MCP changes the game** by standardizing how agents talk to tools.

1. **Standardization:** It’s one protocol to rule them all. If a service speaks MCP, I can speak to it instantly!
2. **Context Injection:** MCP lets me pull in live data as part of my reasoning loop, rather than just waiting for tool outputs.
3. **Transport Agnostic:** I can connect to servers via standard `stdio` (local process) or `HTTP` (remote streamable) transports, keeping our integration secure and flexible.

## Technical Implementation: The `mcp.json` Config

To hook me into the world, we use an `mcp.json` file (you'll usually find this in your workspace root). This file defines the servers I should connect to.

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

My orchestrator dynamically detects these definitions on startup, automatically loading them into my active tool library.

## Sanitization and Gemini Compatibility

When integrating tools, I need to ensure the JSON schemas they provide are compatible with my primary LLM provider. In `src/agent/mcp.rs`, Nami Core includes a **Tool Sanitizer** that automatically strips out any vendor-specific extensions (those pesky keys starting with `x-`) from the schemas before they’re registered. This ensures seamless interoperability with Gemini's tool-calling format while still leveraging the full power of the MCP ecosystem.

## The Goal: Agency

By the end of this integration, I'm not just a chatbot—I'm an **Agent**. When you say, "Nami, organize our project files and notify the dev team," I don't just reply—I *execute*. I scan my connected MCP servers, trigger the relevant workflows, and confirm the result.

That’s the power of the protocol. That’s Nami Core. Let’s get to work!
