mod agent;
mod modes;
mod runner;
mod tools;
mod utils;

use std::sync::Arc;

use adk_telemetry::shutdown_telemetry;
use clap::{Parser, Subcommand};
use runner::AgentRunner;
use modes::startup::setup_dependencies;

/// The command-line interface for the application.
#[derive(Parser)]
#[command(name = "agent-app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Supported commands for the application.
#[derive(Subcommand)]
enum Commands {
    /// Run the bot mode (e.g., Telegram integration).
    Bot,
    /// Run the interactive CLI mode.
    Cli,
    /// Initialize the project configuration and database.
    Init,
    /// Execute a prompt directly and exit.
    Run {
        /// The prompt to execute.
        prompt: String
    },
    /// Start the HTTP server.
    Serve {
        /// Optional port to serve on.
        port: Option<u16>
    },
    /// Browse mode with embedded WebUI.
    Browse {
        /// Optional port for the browser UI.
        port: Option<u16>
    },
    /// Run the LINE bot mode.
    Line {
        /// Optional port for the LINE webhook server.
        port: Option<u16>
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();

    // parse cli
    let cli = Cli::parse();

    if !matches!(cli.command, Commands::Serve { .. } | Commands::Browse { .. } | Commands::Init) {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "info") };
        }
        pretty_env_logger::init();
    }

    if let Commands::Init = cli.command {
        modes::init::initialize_project().await?;
        return Ok(());
    }

    log::info!("Application starting...");

    // shared setup
    log::info!("Building agent...");
    let config = agent::load_config_sync().unwrap_or_else(|e| {
        log::warn!("Failed to load config.toml: {}. Using defaults.", e);
        agent::AppConfig {
            model: agent::ModelConfig {
                provider: Some("gemini".to_string()),
                model_name: "gemini-2.5-flash".to_string(),
                api_key_env: Some("GOOGLE_API_KEY".to_string()),
                base_url: None,
                project_id: None,
                location: None,
            },
            specialists: None,
            image_generation: None,
            reflection: None,
            embedding: None,
        }
    });

    let (agent, model, provider, model_name) = agent::build_agent().await?;
    log::info!("Agent built successfully.");

    let deps = setup_dependencies().await?;

    // Reflection Service
    if config.reflection.as_ref().map(|r| r.enabled).unwrap_or(false) {
        let reflection_model_cfg = config.reflection.as_ref().and_then(|r| r.to_model_config());
        let reflection_model = agent::load_model_with_fallback(&reflection_model_cfg, &config.model).await?;
        let reflection_model_name = reflection_model_cfg
            .as_ref()
            .map(|c| c.model_name.clone())
            .unwrap_or_else(|| config.model.model_name.clone());

        let reflection_svc = Arc::new(agent::reflection::ReflectionService::new(
            reflection_model,
            reflection_model_name,
            deps.sessions.clone(),
            deps.memory.clone(),
        ));
        tokio::spawn(async move {
            reflection_svc.start().await;
        });
    }

    match cli.command {
        Commands::Bot => {
            log::info!("Running in bot mode");
            let runner = Arc::new(AgentRunner::new(
                agent,
                deps.sessions.clone(),
                deps.memory_adapter.clone(),
                "telegram",
                model,
            ));
            modes::bot::run_bot(runner, deps.sessions.clone()).await?;
        }
        Commands::Cli => {
            log::info!("Running in CLI mode");
            modes::cli::run_cli(agent, deps.sessions, model, provider, model_name).await?;
        }
        Commands::Run { prompt } => {
            log::info!("Running in direct run mode");
            modes::run::run_direct(agent, &prompt).await?;
        }
        Commands::Serve { port } => {
            log::info!("Running in serve mode");
            modes::serve::run_serve(agent, model, deps.memory_adapter, port.unwrap_or(8080)).await?;
        }
        Commands::Browse { port } => {
            log::info!("Running in browse mode");
            modes::browse::run_browse(agent, model, deps.memory_adapter, port.unwrap_or(8080)).await?;
        }
        Commands::Line { port } => {
            log::info!("Running in LINE bot mode");
            let runner = Arc::new(AgentRunner::new(
                agent,
                deps.sessions.clone(),
                deps.memory_adapter,
                "line",
                model,
            ));
            modes::line::run_line(runner, port.unwrap_or(8080)).await?;
        }
        Commands::Init => unreachable!(),
    }

    // shutdown telemetry
    if !std::env::var("OTEL_COLLECTOR").unwrap_or_default().is_empty() {
        shutdown_telemetry();
    }
    Ok(())
}
