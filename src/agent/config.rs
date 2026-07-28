use adk_rust::model::{OpenAIClient, OpenAIConfig};
use adk_rust::prelude::*;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::utils::get_nami_dir;

/// Application configuration structure loaded from `config.toml`.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AppConfig {
    /// LLM provider configuration.
    pub model: ModelConfig,
    /// Optional configuration for specialized agents.
    pub specialists: Option<SpecialistsConfig>,
    /// Optional configuration for image generation.
    pub image_generation: Option<ModelConfig>,
    /// Optional configuration for audio generation.
    pub audio_generation: Option<ModelConfig>,
    /// Optional configuration for video generation.
    pub video_generation: Option<ModelConfig>,
    /// Optional configuration for reflection service.
    pub reflection: Option<ReflectionConfig>,
    /// Optional configuration for embedding service.
    pub embedding: Option<ModelConfig>,
    /// Optional configuration for tools, like shell command whitelist.
    pub tools: Option<ToolsConfig>,
}

/// Configuration for tools (e.g., shell commands whitelist).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToolsConfig {
    pub shell: Option<ShellToolConfig>,
    pub enabled_categories: Option<Vec<String>>,
}

/// Configuration for the shell tool.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ShellToolConfig {
    pub allowed_commands: Option<Vec<String>>,
    pub blocked_commands: Option<Vec<String>>,
    pub security_level: Option<String>,
    pub sanitize_environment: Option<bool>,
}

/// Configuration details for the LLM provider and specific model.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ModelConfig {
    /// Name of the LLM provider (e.g., "gemini", "anthropic", "vertex", "openai").
    pub provider: Option<String>,
    /// Identifier for the model to use (e.g., "gemini-1.5-pro", "gpt-4o").
    pub model_name: String,
    /// Environment variable name containing the API key for this provider.
    pub api_key_env: Option<String>,
    /// Optional base URL for API requests, useful for compatible providers or local proxies.
    pub base_url: Option<String>,
    /// Google Cloud Project ID, required for Vertex AI.
    pub project_id: Option<String>,
    /// Google Cloud Location (region), required for Vertex AI.
    pub location: Option<String>,
}

/// Configuration for dynamic custom specialist agents.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CustomSpecialistConfig {
    pub provider: Option<String>,
    pub model_name: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub description: String,
    pub instruction: String,
    pub tools: Option<Vec<String>>,
    pub workspace_mode: Option<String>,
}

/// Configuration for individual specialized agents.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SpecialistsConfig {
    /// Configuration for the coding specialist.
    pub coder: Option<ModelConfig>,
    /// Configuration for the research specialist.
    pub researcher: Option<ModelConfig>,
    /// Configuration for the writing specialist.
    pub writer: Option<ModelConfig>,
    /// Configuration for the Ralph Wiggum (recursive) agent.
    pub ralph: Option<ModelConfig>,
    /// Configuration for the generalist agent.
    pub generalist: Option<ModelConfig>,
    /// Additional dynamic custom specialist agents.
    #[serde(default)]
    pub custom: Option<HashMap<String, CustomSpecialistConfig>>,
}

impl SpecialistsConfig {
    /// Helper to cleanly load all specialist LLM instances.
    pub async fn load_all_models(
        &self,
        default_config: &ModelConfig,
    ) -> anyhow::Result<HashMap<String, Arc<dyn Llm>>> {
        let mut models = HashMap::new();
        let configs = [
            ("coder", &self.coder),
            ("researcher", &self.researcher),
            ("writer", &self.writer),
            ("ralph", &self.ralph),
            ("generalist", &self.generalist),
        ];

        for (name, cfg) in configs {
            if let Some(cfg) = cfg {
                let model = load_model_with_fallback(&Some(cfg.clone()), default_config).await?;
                models.insert(name.to_string(), model);
            }
        }

        if let Some(ref custom_specs) = self.custom {
            for (name, custom_cfg) in custom_specs {
                let model_cfg = ModelConfig {
                    provider: custom_cfg.provider.clone(),
                    model_name: custom_cfg
                        .model_name
                        .clone()
                        .unwrap_or_else(|| default_config.model_name.clone()),
                    api_key_env: custom_cfg.api_key_env.clone(),
                    base_url: custom_cfg.base_url.clone(),
                    project_id: custom_cfg.project_id.clone(),
                    location: custom_cfg.location.clone(),
                };
                let loaded_model =
                    load_model_with_fallback(&Some(model_cfg), default_config).await?;
                models.insert(name.clone(), loaded_model);
            }
        }

        Ok(models)
    }
}

/// Configuration for the reflection service.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ReflectionConfig {
    /// Whether the reflection service is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Name of the LLM provider for the reflection service.
    pub provider: Option<String>,
    /// Identifier for the model to use for reflection.
    pub model_name: Option<String>,
    /// Environment variable name containing the API key.
    pub api_key_env: Option<String>,
    /// Optional base URL for the reflection service.
    pub base_url: Option<String>,
    /// Google Cloud Project ID for Vertex AI reflection.
    pub project_id: Option<String>,
    /// Google Cloud Location for Vertex AI reflection.
    pub location: Option<String>,
}

impl ReflectionConfig {
    /// Converts the reflection configuration into a standard `ModelConfig`.
    pub fn to_model_config(&self) -> Option<ModelConfig> {
        self.model_name.as_ref().map(|name| ModelConfig {
            provider: self.provider.clone(),
            model_name: name.clone(),
            api_key_env: self.api_key_env.clone(),
            base_url: self.base_url.clone(),
            project_id: self.project_id.clone(),
            location: self.location.clone(),
        })
    }
}

/// Synchronously loads the application configuration from `~/.nami/config.toml`.
pub fn load_config_sync() -> anyhow::Result<AppConfig> {
    let config_path = get_nami_dir().join("config.toml");
    let config_str = std::fs::read_to_string(config_path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}

/// Synchronously saves the application configuration to `~/.nami/config.toml`.
pub fn save_config_sync(config: &AppConfig) -> anyhow::Result<()> {
    let config_path = get_nami_dir().join("config.toml");
    let config_str = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, config_str.as_bytes())?;
    Ok(())
}

/// Loads and initializes an LLM instance based on the provided configuration.
///
/// For most providers, it fetches an API key from an environment variable.
/// For the "vertex" provider, it initializes using Application Default Credentials (ADC).
pub async fn load_model(model_config: &ModelConfig) -> anyhow::Result<Arc<dyn Llm>> {
    let provider = model_config.provider.as_deref().unwrap_or("gemini");

    // Vertex AI uses Application Default Credentials (ADC) and doesn't require an explicit API key.
    if provider == "vertex" {
        return Ok(Arc::new(GeminiModel::new_google_cloud_adc(
            model_config.project_id.as_deref().unwrap_or_default(),
            model_config.location.as_deref().unwrap_or_default(),
            &model_config.model_name,
        )?));
    }

    let api_key_env = model_config
        .api_key_env
        .as_deref()
        .unwrap_or("GOOGLE_API_KEY");
    let api_key = std::env::var(api_key_env)
        .with_context(|| format!("Environment variable {} not set", api_key_env))?;

    match provider {
        "gemini" => Ok(Arc::new(GeminiModel::new(
            &api_key,
            &model_config.model_name,
        )?)),
        "anthropic" => Ok(Arc::new(AnthropicClient::new(AnthropicConfig::new(
            &api_key,
            &model_config.model_name,
        ))?)),
        "openrouter" => Ok(Arc::new(OpenRouterClient::new(OpenRouterConfig::new(
            &api_key,
            &model_config.model_name,
        ))?)),
        "openai" => {
            let config = if let Some(url) = &model_config.base_url {
                OpenAIConfig::compatible(&api_key, url, &model_config.model_name)
            } else {
                OpenAIConfig::new(&api_key, &model_config.model_name)
            };
            Ok(Arc::new(OpenAIClient::new(config)?))
        }
        "ollama" => Ok(Arc::new(OllamaModel::new(OllamaConfig::new(
            &model_config.model_name,
        ))?)),
        "thaillm" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::compatible(
            &api_key,
            "https://thaillm.or.th/api/v1",
            &model_config.model_name,
        ))?)),
        _ => anyhow::bail!("Unsupported provider: {}", provider),
    }
}

/// Loads a model using a specific configuration, falling back to default values for missing fields.
pub async fn load_model_with_fallback(
    specific: &Option<ModelConfig>,
    default: &ModelConfig,
) -> anyhow::Result<Arc<dyn Llm>> {
    match specific {
        Some(config) => {
            if config.provider.is_some()
                && config.api_key_env.is_some()
                && config.base_url.is_some()
                && config.project_id.is_some()
                && config.location.is_some()
            {
                return load_model(config).await;
            }
            let mut effective_config = config.clone();
            if effective_config.provider.is_none() {
                effective_config.provider = default.provider.clone();
            }
            if effective_config.api_key_env.is_none() {
                effective_config.api_key_env = default.api_key_env.clone();
            }
            if effective_config.base_url.is_none() {
                effective_config.base_url = default.base_url.clone();
            }
            if effective_config.project_id.is_none() {
                effective_config.project_id = default.project_id.clone();
            }
            if effective_config.location.is_none() {
                effective_config.location = default.location.clone();
            }
            load_model(&effective_config).await
        }
        None => load_model(default).await,
    }
}

pub async fn load_optional_model(
    cfg: &Option<ModelConfig>,
    default: &ModelConfig,
) -> anyhow::Result<Option<Arc<dyn Llm>>> {
    match cfg {
        Some(c) => Ok(Some(
            load_model_with_fallback(&Some(c.clone()), default).await?,
        )),
        None => Ok(None),
    }
}
