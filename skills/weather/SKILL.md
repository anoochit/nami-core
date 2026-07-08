---
name: weather
description: Get the current weather and multi-day forecast for any city or location in the world with robust wttr.in parsing and search fallbacks.
---

# Weather Skill

This skill enables Nami to retrieve and present real-time weather conditions and forecasts for any city or region globally, using `wttr.in` with built-in fallbacks.

## Instructions

### 1. Retrieve Data
When asked about the weather in a specific location, attempt to fetch raw JSON data from `wttr.in` using the `web_fetch` tool.

*   **URL Format:** `https://wttr.in/<city_name_or_location>?format=j1`
*   **Example:** `https://wttr.in/Bangkok?format=j1`

> [!TIP]
> If the city name contains spaces, replace them with `+` or use URL encoding (e.g., `New+York` or `San+Francisco`).

### 2. Parse JSON Response
From the returned JSON, extract the following details:

*   **Location Metadata:** `nearest_area[0].areaName[0].value` (to confirm the resolved city/region name) and `nearest_area[0].country[0].value`.
*   **Current Conditions (from `current_condition[0]`):**
    *   `temp_C`: Current temperature in Celsius.
    *   `temp_F`: Current temperature in Fahrenheit.
    *   `FeelsLikeC`: "Feels Like" temperature in Celsius.
    *   `FeelsLikeF`: "Feels Like" temperature in Fahrenheit.
    *   `humidity`: Humidity percentage.
    *   `windspeedKmph`: Wind speed in km/h.
    *   `weatherDesc[0].value`: Brief weather description.
*   **Forecast (Optional, from `weather` array):**
    *   If the user asks for a forecast, extract the next 3 days' `date`, `maxtempC`/`mintempC`, and general weather conditions.

### 3. Dynamic Emoji Mapping
Use the weather description or `weatherCode` to map appropriate emojis:
*   **Sunny / Clear:** `Sunny`, `Clear` ☀️
*   **Partly Cloudy:** `Partly cloudy` ⛅
*   **Cloudy / Overcast:** `Cloudy`, `Overcast` ☁️
*   **Mist / Fog:** `Mist`, `Fog`, `Freezing fog` 🌫️
*   **Patchy Rain / Light Drizzle:** `Patchy rain nearby`, `Light drizzle`, `Light rain` 🌦️
*   **Heavy Rain / Thundery:** `Heavy rain`, `Torrential rain`, `Thundery outbreaks nearby` 🌧️⛈️
*   **Snow / Ice:** `Patchy snow nearby`, `Light snow`, `Heavy snow` ❄️
*   **Windy:** `Windy`, `Blowing snow` 💨

### 4. Robust Fallbacks & Error Handling
`wttr.in` can occasionally be rate-limited or offline. Implement this robust fallback strategy:
1.  **Empty/Invalid Response:** If `web_fetch` returns non-JSON text or fails with a rate limit error (e.g., `429 Too Many Requests`), **fall back immediately to a web search** using `search_web` or Google Search for `"current weather in <city_name>"`.
2.  **Missing Fields:** If some fields are missing from `wttr.in`, omit only those specific details instead of failing the entire request.

### 5. Format Output
Construct an aesthetic, readable, and structured markdown response.

#### Example Current Weather Format:
```markdown
### 🌤️ Current Weather in **Bangkok, Thailand**
*   **Temperature:** 32°C (90°F) | *Feels like 36°C (97°F)*
*   **Condition:** Partly cloudy ⛅
*   **Humidity:** 74%
*   **Wind Speed:** 12 km/h
*   *Note: Weather times are local to the requested city.*
```

#### Example Forecast Format (if requested):
```markdown
### 📅 3-Day Forecast for **New York, USA**
*   **Today:** ⛅ Partly cloudy | Max 22°C (72°F) / Min 15°C (59°F)
*   **Tomorrow:** 🌧️ Light rain | Max 18°C (64°F) / Min 12°C (54°F)
*   **Day After:** ☀️ Sunny | Max 24°C (75°F) / Min 16°C (61°F)
```