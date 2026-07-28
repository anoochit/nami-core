use crate::utils::paths::{get_nami_dir, get_http_client};

pub fn save_agent_statistic(provider: &str, model_name: &str, duration_secs: f64, total_tokens: usize) {
    let stats_path = get_nami_dir().join("stats.json");

    let new_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "provider": provider,
        "model_name": model_name,
        "duration_seconds": duration_secs,
        "tokens_consumed": total_tokens,
    });

    let mut stats_list = if stats_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&stats_path) {
            serde_json::from_str::<Vec<serde_json::Value>>(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    stats_list.push(new_entry);

    if let Ok(serialized) = serde_json::to_string_pretty(&stats_list) {
        let _ = std::fs::write(&stats_path, serialized);
    }
}

pub async fn fetch_models_for_provider(provider: &str) -> Vec<String> {
    let fetched = match provider {
        "ollama" => fetch_ollama_models().await,
        "thaillm" => None,
        _ => fetch_openrouter_models(provider).await,
    };

    match fetched {
        Some(mut models) if !models.is_empty() => {
            models.sort();
            if models.last().map(|s| s.as_str()) != Some("custom") {
                models.push("custom".to_string());
            }
            models
        }
        _ => fallback_models(provider),
    }
}

async fn fetch_openrouter_models(provider: &str) -> Option<Vec<String>> {
    let prefix = match provider {
        "gemini" | "vertex" => "google/",
        "openai" => "openai/",
        "anthropic" => "anthropic/",
        "openrouter" => "",
        _ => return None,
    };

    let client = get_http_client();
    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .header("User-Agent", "nami-cli")
        .send()
        .await
        .ok()?;

    let body: serde_json::Value = resp.json().await.ok()?;
    let data = body.get("data")?.as_array()?;

    let mut models: Vec<String> = data
        .iter()
        .filter_map(|m| m.get("id")?.as_str().map(|s| s.to_string()))
        .filter(|id| prefix.is_empty() || id.starts_with(prefix))
        .map(|id| {
            if prefix.is_empty() || !id.starts_with(prefix) {
                id
            } else {
                id[prefix.len()..].to_string()
            }
        })
        .collect();

    models.sort();
    if prefix.is_empty() {
        models.truncate(50);
    }
    Some(models)
}

async fn fetch_ollama_models() -> Option<Vec<String>> {
    let client = get_http_client();
    let resp = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .ok()?;

    let body: serde_json::Value = resp.json().await.ok()?;
    let models_list = body.get("models")?.as_array()?;

    let mut models: Vec<String> = models_list
        .iter()
        .filter_map(|m| m.get("name")?.as_str().map(|s| s.to_string()))
        .collect();

    models.sort();
    Some(models)
}

fn fallback_models(provider: &str) -> Vec<String> {
    let mut result: Vec<String> = match provider {
        "anthropic" => vec![
            "claude-opus-4-6".into(),
            "claude-sonnet-4-6".into(),
            "claude-haiku-4-5-20251001".into(),
            "claude-opus-4-5-20251101".into(),
            "claude-sonnet-4-5-20250929".into(),
        ],
        "gemini" | "vertex" => vec![
            "gemini-pro-latest".into(),
            "gemini-flash-latest".into(),
            "gemini-3.1-pro-preview".into(),
            "gemini-3-flash-preview".into(),
            "gemini-2.5-pro".into(),
            "gemini-2.5-flash".into(),
        ],
        "ollama" => vec!["deepseek-r1:1.5b".into()],
        "openai" => vec!["gpt-5".into(), "gpt-4.1".into()],
        "openrouter" => vec![
            "anthropic/claude-3.5-sonnet".into(),
            "tencent/hy3-preview:free".into(),
            "openrouter/free".into(),
        ],
        "thaillm" => vec![
            "openthaigpt-thaillm-8b-instruct-v7.2".into(),
            "pathumma-thaillm-qwen3-8b-think-3.0.0".into(),
            "typhoon-s-thaillm-8b-instruct".into(),
            "thalle-0.2-thaillm-8b-fa".into(),
        ],
        _ => vec![],
    };
    result.push("custom".to_string());
    result
}