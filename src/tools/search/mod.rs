use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    /// The search query.
    query: String,
}

/// Searches Google for information using the Serper.dev API.
#[tool]
async fn google_search(args: SearchArgs) -> std::result::Result<Value, AdkError> {
    let api_key = std::env::var("SERPER_API_KEY")
        .map_err(|_| AdkError::tool("SERPER_API_KEY not found in environment"))?;

    let client = crate::utils::get_http_client().clone();
    let response = client
        .post("https://google.serper.dev/search")
        .header("X-API-KEY", api_key)
        .header("Content-Type", "application/json")
        .json(&json!({
            "q": args.query,
            "num": 5
        }))
        .send()
        .await
        .map_err(|e| AdkError::tool(format!("Search request failed: {}", e)))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to parse search results: {}", e)))?;

    Ok(data)
}

pub fn search_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(GoogleSearch)]
}
