---
name: weather
description: Get the current weather for any city or location in the world.
---

# Weather Skill

This skill allows Nami to fetch real-time weather information for any city or region.

## Instructions
1. When asked about the weather in a specific city, use the `web_fetch` tool (or run a `curl` command) to fetch JSON formatting from wttr.in:
   ```
   https://wttr.in/<city_name>?format=j1
   ```
2. Parse the JSON response. Extract:
   - Current temperature (`temp_C` and `temp_F`)
   - Feels Like temperature (`FeelsLikeC`)
   - Humidity (`humidity`)
   - Wind Speed (`windspeedKmph`)
   - Weather Description (`weatherDesc`)
3. Format the weather update into a beautiful, user-friendly message in the chat with matching emojis (e.g., ☀️, 🌧️, ❄️, 💨).
