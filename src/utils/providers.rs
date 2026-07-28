pub fn provider_env_var(provider: &str) -> &'static str {
    match provider {
        "gemini" | "vertex" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "ollama" => "",
        "thaillm" => "THAILLM_API_KEY",
        _ => "",
    }
}

pub fn default_models(provider: &str) -> Vec<&'static str> {
    match provider {
        "gemini" | "vertex" => vec![
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "gemini-3.1-pro-preview",
        ],
        "openai" => vec![
            "gpt-4.1",
            "gpt-5",
        ],
        "anthropic" => vec![
            "claude-sonnet-4-6",
            "claude-haiku-4-5-20251001",
        ],
        "ollama" => vec!["deepseek-r1:1.5b"],
        "openrouter" => vec!["anthropic/claude-3.5-sonnet"],
        "thaillm" => vec!["openthaigpt-thaillm-8b-instruct-v7.2"],
        _ => vec![],
    }
}
