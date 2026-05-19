use std::sync::Arc;
use std::time::Duration;
use adk_telemetry::{init_with_otlp, shutdown_telemetry};
use clap::{Parser, Subcommand};
use nami::runner::AgentRunner;
use nami::agent;
use nami::modes;
use nami::modes::init::run_init;
use nami::modes::startup::setup_dependencies;


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
    /// Run the interactive TUI mode (Ratatui).
    Tui,
    /// Initialize the project configuration and database.
    Init,
    /// Run automated evaluations against a dataset.
    Eval,
    /// Execute a prompt directly and exit.
    Run {
        /// The prompt to execute.
        prompt: String
    },
    /// Start the HTTP server.
    Serve {
        /// Optional port to serve on.
        port: Option<u16>,
        /// Optional host to listen on (defaults to 127.0.0.1).
        #[arg(long)]
        host: Option<String>,
    },
    /// Browse mode with embedded WebUI.
    Browse {
        /// Optional port for the browser UI.
        port: Option<u16>,
        /// Optional host to listen on (defaults to 127.0.0.1).
        #[arg(long)]
        host: Option<String>,
    },
    /// Run the LINE bot mode.
    Line {
        /// Optional port for the LINE webhook server.
        port: Option<u16>,
        /// Optional host to listen on (defaults to 127.0.0.1).
        #[arg(long)]
        host: Option<String>,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Application entry reached");
    log::info!("Application entry reached");
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();

    println!("Parsing CLI...");
    let cli = Cli::parse();

    // Logging & Telemetry setup
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    println!("DEBUG: RUST_LOG='{}'", rust_log);
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info") };
    }

    let otel_endpoint = std::env::var("OTEL_COLLECTOR").unwrap_or_else(|_| "NOT_SET".to_string());
    let use_telemetry = otel_endpoint != "NOT_SET" && !otel_endpoint.is_empty();
    println!("DEBUG: OTEL_COLLECTOR='{}'", otel_endpoint);
    
    println!("Initializing telemetry...");
    if use_telemetry {
        init_with_otlp("nami", &otel_endpoint).expect("Failed to initialize telemetry");
    }

    println!("Telemetry initialized. Starting app...");
    log::info!("Application starting with telemetry: {}", use_telemetry);
    tracing::info!("Application starting with telemetry: {}", use_telemetry);

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

    // Spawn scheduler background loop
    let agent_scheduler = agent.clone();
    let sessions_scheduler = deps.sessions.clone();
    let model_scheduler = model.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::modes::scheduler::run_scheduler_loop_with_deps(agent_scheduler, model_scheduler, sessions_scheduler).await {
            log::error!("Scheduler background error: {:?}", e);
        }
    });

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
            unsafe { std::env::set_var("RUST_LOG", "error") };
            modes::cli::run_cli(agent, deps.sessions, model, provider, model_name).await?;
        }
        Commands::Tui => {
            unsafe { std::env::set_var("RUST_LOG", "error") };
            modes::tui::run_tui(agent, deps.sessions, deps.memory_adapter, model, provider, model_name).await?;
        }
        Commands::Run { prompt } => {
            log::info!("Running in direct run mode");
            modes::run::run_direct(agent, &prompt).await?;
        }
        Commands::Serve { port, host } => {
            log::info!("Running in serve mode");
            let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
            modes::serve::run_serve(agent, model, deps.sessions.clone(), deps.memory_adapter, host, port.unwrap_or(8080)).await?;
        }
        Commands::Browse { port, host } => {
            log::info!("Running in browse mode");
            let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
            modes::browse::run_browse(agent, model, deps.memory_adapter, host, port.unwrap_or(8080)).await?;
        }
        Commands::Line { port, host } => {
            log::info!("Running in LINE bot mode");
            let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
            let runner = Arc::new(AgentRunner::new(
                agent,
                deps.sessions.clone(),
                deps.memory_adapter,
                "line",
                model,
            ));
            modes::line::run_line(runner, host, port.unwrap_or(8080)).await?;
        }
        Commands::Init => {
            log::info!("Running initialize mode");
            run_init().await?;
        },
        Commands::Eval => {
            log::info!("Running evaluation mode");
            modes::eval::run_eval(agent, deps.sessions, deps.memory_adapter, model).await?;
        }
    }

    // shutdown telemetry
    if use_telemetry {
        log::info!("Flushing telemetry...");
        tokio::time::sleep(Duration::from_millis(500)).await;
        shutdown_telemetry();
    }
    Ok(())
}
