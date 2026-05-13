use std::sync::Arc;

use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
struct WebFetchArgs {
    /// The URL to fetch data from.
    url: String,
}

/// Fetch content from a URL via HTTP GET request. Use this tool whenever you need to access information from a website, summarize a web page, or retrieve data from a given URL.
#[tool]
async fn web_fetch(args: WebFetchArgs) -> std::result::Result<Value, AdkError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) adk-rust-bot/1.0")
        .build()
        .map_err(|e| AdkError::tool(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .get(&args.url)
        .send()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to fetch URL: {}", e)))?;

    let status = response.status();

    // Read response body as text
    let text = response
        .text()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read response body: {}", e)))?;

    // Truncate text if it's too large to avoid overwhelming the LLM context (e.g. max 50000 chars)
    let max_len = 50000;
    let content = if text.len() > max_len {
        format!(
            "{}... (truncated, original length: {})",
            &text[..max_len],
            text.len()
        )
    } else {
        text
    };

    Ok(json!({
        "status": status.as_u16(),
        "url": args.url,
        "content": content
    }))
}

pub fn web_fetch_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(WebFetch)]
}
