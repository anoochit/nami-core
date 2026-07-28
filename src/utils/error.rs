#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCategory {
    Transient,
    Fatal,
}

pub fn categorize_error(e: &anyhow::Error) -> ErrorCategory {
    let err_str = e.to_string().to_lowercase();
    if err_str.contains("rate_limited")
        || err_str.contains("429")
        || err_str.contains("timeout")
        || err_str.contains("408")
        || err_str.contains("503")
        || err_str.contains("529")
        || (err_str.contains("400") && err_str.contains("number of function response parts"))
    {
        ErrorCategory::Transient
    } else {
        ErrorCategory::Fatal
    }
}

pub fn clean_error_message(e: impl std::fmt::Display) -> String {
    let err_str = e.to_string();

    if err_str.contains("insufficient_quota") {
        return "API Quota Exceeded: You have exceeded your OpenAI quota. Please check your plan and billing details.".to_string();
    }

    if err_str.contains("rate_limited") || err_str.contains("429 Too Many Requests") {
        return "Rate Limit Reached: The AI provider is currently rate limiting requests. Please wait a moment before trying again.".to_string();
    }

    if err_str.contains("invalid_api_key") || err_str.contains("401 Unauthorized") {
        return "Invalid API Key: The API key provided is invalid or has expired. Please check your configuration.".to_string();
    }

    let mut clean_msg = err_str.clone();

    if let Some(json_start) = err_str.find('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&err_str[json_start..]) {
            if let Some(msg) = v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                clean_msg = msg.to_string();
            } else if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                clean_msg = msg.to_string();
            }
        }
    }

    if clean_msg.contains("error=") {
        if let Some(idx) = clean_msg.rfind("error=") {
            clean_msg = clean_msg[idx + 6..].to_string();
        }
    }

    if let Some(idx) = clean_msg.find("): {") {
        clean_msg = clean_msg[..idx].to_string();
    }

    clean_msg.trim().to_string()
}