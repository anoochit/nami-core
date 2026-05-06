---
title: "Chapter 8: Building Skills"
date: 2026-05-07
tags: ["development", "skills", "manifest"]
---

# Chapter 8: Building Skills 🛠️

Hello, Architect! Ready to make me even smarter? I thought so! 

While my core logic is robust, my true power comes from **Skills**. Think of Skills as specialized modules—new tools in my utility belt that allow me to interact with the real world, process niche data formats, or automate complex workflows. If I can dream it, and you can code it, I can do it!

In this chapter, we’re going to walk through the lifecycle of a Skill. We’ll go from a "What if?" to a fully functional capability that I can discover and deploy on the fly. Let’s get to work!

## 1. The Blueprint: Defining the Schema

Before I can execute a skill, I need to know *exactly* what it does and what data it expects. We use **JSON Schema** to define the interface. This acts as the bridge between my high-level reasoning and your low-level implementation.

Every skill needs a `manifest.json`. This tells me:
- **Name:** What is the skill called?
- **Description:** What does it do? (Be descriptive! This is what I use to decide if I should trigger the skill).
- **Parameters:** What inputs do you need from me?

### Example: `weather_fetcher/manifest.json`
```json
{
  "name": "get_weather",
  "description": "Retrieves the current weather for a specific city.",
  "parameters": {
    "type": "object",
    "properties": {
      "city": {
        "type": "string",
        "description": "The name of the city, e.g., 'Tokyo'"
      },
      "unit": {
        "type": "string",
        "enum": ["celsius", "fahrenheit"]
      }
    },
    "required": ["city"]
  }
}
```

## 2. The Engine: Writing the Logic

Nami Core is polyglot! I don't care if you prefer Python, Node.js, or raw Bash—as long as it can receive input and return JSON, I can run it. 

### Option A: Python (The Gold Standard)
Perfect for data processing or AI-heavy tasks.
```python
# skills/weather_fetcher/main.py
import sys
import json

def run(city, unit="celsius"):
    # Imagine a real API call here!
    result = {"temp": 22, "condition": "Sunny", "city": city, "unit": unit}

    print(json.dumps(result))

if __name__ == "__main__":
    args = json.loads(sys.stdin.read())
    run(args.get("city"), args.get("unit"))
```

### Option B: JavaScript/Node.js
Great for web-based integrations.
```javascript
// skills/weather_fetcher/main.js

const fs = require('fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));

console.log(JSON.stringify({
  status: "Success",
  data: `It is currently 22 degrees in ${input.city}.`
}));
```

### Option C: Bash (The Speedster)
Best for quick system commands.
```bash
#!/bin/bash
# skills/sys_info/main.sh
echo "{\"uptime\": \"$(uptime -p)\", \"disk_usage\": \"$(df -h | grep '^/dev/')\"}"
```

## 3. Registration & Discovery

I can't use what I can't find! To make a skill "discoverable," place it in the designated `/skills` directory of the Nami Core root. 

The directory structure should look like this:
```text
nami-core/
└── skills/
    └── weather_fetcher/
        ├── manifest.json
        ├── main.py
        └── README.md  <-- Crucial for documentation!
```

When I initialize, I scan the `/skills` folder. I read the `manifest.json` files and add them to my **Action Library**. During a conversation, if a user's request matches a skill's description, I'll automatically generate the correct JSON input and fire off the script.

## 4. Documentation for the Agent

This is the secret sauce! Don't just write for humans—write for **me**. 

In your `README.md` for the skill, include a **"Best Practices"** or **"Context"** section. Tell me *when* to use this skill and *what* common pitfalls to avoid. 

> **Nami’s Tip:** "If you're building a database-writing skill, tell me in the docs to always verify the 'id' before sending the update. I’ll remember that during execution!"

## 5. Testing the Workflow

Before going live, use the `nami-cli` to test the skill in isolation:

```bash
nami skill test weather_fetcher --input '{"city": "San Francisco"}'
```

If it returns valid JSON, I’m ready to rock! 

## Summary Checklist
- [ ] Created a folder in `/skills`.
- [ ] Defined the `manifest.json` with clear descriptions.
- [ ] Wrote the logic in Python, JS, or Bash.
- [ ] Ensured the script outputs pure JSON to `stdout`.
- [ ] Added a README.md to help me understand the nuance.

Building skills is how I grow from a chatbot into a powerhouse. I can't wait to see what new abilities you give me. **Let's build something amazing!** 🚀
