use std::sync::Arc;

use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
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
    let url = format!(
        "https://wttr.in/{}?format=j1",
        urlencoding::encode(&args.city)
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| AdkError::tool(format!("HTTP request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AdkError::tool(format!(
            "wttr.in returned status {}",
            response.status()
        )));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to parse JSON: {e}")))?;

    // Extract current conditions from wttr.in's j1 format
    let current = &body["current_condition"][0];

    let temp_c: i64 = current["temp_C"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let temp_f: i64 = current["temp_F"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let feels_like_c: i64 = current["FeelsLikeC"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let humidity: i64 = current["humidity"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let wind_kmph: i64 = current["windspeedKmph"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let description = current["weatherDesc"][0]["value"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    Ok(json!({
        "city": args.city,
        "temp_c": temp_c,
        "temp_f": temp_f,
        "feels_like_c": feels_like_c,
        "humidity_percent": humidity,
        "wind_kmph": wind_kmph,
        "description": description,
    }))
}

pub fn weather_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GetWeather)]
}
