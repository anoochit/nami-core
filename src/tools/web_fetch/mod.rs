use std::sync::{Arc, OnceLock, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use reqwest::Client;
use schemars::JsonSchema;
use serde_json::{Value, json};

// Built once, reused across every call — avoids TLS handshake setup per request
static CLIENT: OnceLock<Client> = OnceLock::new();

struct CacheEntry {
    result: Value,
    cached_at: Instant,
}

static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();

fn get_client() -> Result<&'static Client, AdkError> {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) adk-rust-bot/1.0")
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("Failed to build HTTP client")
    });
    CLIENT.get().ok_or_else(|| AdkError::tool("HTTP client unavailable"))
}

fn get_cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Anything that isn't text is useless to an LLM — skip reading the body entirely
fn is_binary(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    ct.starts_with("image/")
        || ct.starts_with("audio/")
        || ct.starts_with("video/")
        || ct.starts_with("application/octet-stream")
        || ct.starts_with("application/zip")
        || ct.starts_with("application/pdf")
}

#[derive(Deserialize, JsonSchema)]
struct WebFetchArgs {
    /// The URL to fetch data from.
    url: String,
    /// Maximum characters to return (default 50000, max 200000).
    max_chars: Option<usize>,
    /// Bypass cache and force a fresh fetch
    bypass_cache: Option<bool>,
}

/// Fetch content from a URL via HTTP GET with intelligent TTL caching. Use when you need to access a website,
/// summarize a page, or retrieve data from a URL.
#[tool]
async fn web_fetch(args: WebFetchArgs) -> std::result::Result<Value, AdkError> {
    let max_len = args.max_chars.unwrap_or(50_000).min(200_000);
    let bypass = args.bypass_cache.unwrap_or(false);

    let cache_key = format!("{}:{}", args.url, max_len);

    // 1. Try checking the TTL cache (5-minute TTL)
    if !bypass {
        if let Ok(cache) = get_cache().lock() {
            if let Some(entry) = cache.get(&cache_key) {
                if entry.cached_at.elapsed() < Duration::from_secs(300) {
                    let mut cached_res = entry.result.clone();
                    cached_res["cached"] = json!(true);
                    cached_res["ttl_remaining_secs"] = json!(300 - entry.cached_at.elapsed().as_secs());
                    return Ok(cached_res);
                }
            }
        }
    }

    // 2. Perform the actual fresh fetch if not cached or bypassed
    let client = get_client()?;
    let response = client
        .get(&args.url)
        .send()
        .await
        .map_err(|e| AdkError::tool(format!("Request failed: {e}")))?;

    let status = response.status().as_u16();

    // Capture content-type before consuming the response
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Early exit — don't waste memory reading binary blobs
    if is_binary(&content_type) {
        let res = json!({
            "status": status,
            "url": args.url,
            "content_type": content_type,
            "error": "Binary content type — not readable as text"
        });
        return Ok(res);
    }

    // Read as raw bytes so we control the UTF-8 boundary ourselves
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read body: {e}")))?;

    // Lossy decode: replaces invalid UTF-8 sequences rather than erroring
    let text = String::from_utf8_lossy(&bytes);

    // Truncate at a valid char boundary (not a byte boundary — avoids broken multibyte chars)
    let (content, truncated) = if text.chars().count() > max_len {
        let boundary = text
            .char_indices()
            .nth(max_len)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        (&text[..boundary], true)
    } else {
        (text.as_ref(), false)
    };

    let mut result = json!({
        "status": status,
        "url": args.url,
        "content_type": content_type,
        "content": content,
        "cached": false,
    });

    if truncated {
        result["truncated"] = json!(true);
        result["original_bytes"] = json!(bytes.len());
    }

    // 3. Cache the new result
    if let Ok(mut cache) = get_cache().lock() {
        cache.insert(cache_key, CacheEntry {
            result: result.clone(),
            cached_at: Instant::now(),
        });
    }

    Ok(result)
}

pub fn web_fetch_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(WebFetch)]
}