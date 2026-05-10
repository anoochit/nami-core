mod agent;
mod modes;
mod runner;
mod tools;
mod utils;

use std::sync::Arc;

use adk_memory::SqliteMemoryService;
use adk_session::SqliteSessionService;
use adk_telemetry::{init_with_otlp, shutdown_telemetry};
use clap::{Parser, Subcommand};
use runner::AgentRunner;

#[derive(Parser)]
#[command(name = "agent-app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bot,                         // namiClaw
    Cli,                         // command line interface
    Init,                        // initialize project files
    Run { prompt: String },      // direct execution
    Serve { port: Option<u16> }, // http server
    Browse { port: Option<u16> }, // server with embedded UI
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // with Otel collector
    let otel_endpoint = std::env::var("OTEL_COLLECTOR").unwrap_or_default();

    if !otel_endpoint.is_empty() {
        log::info!("Init telemetry...");
        init_with_otlp("agent", &otel_endpoint).expect("Failed to initialize telemetry");
    }

    // parse cli
    let cli = Cli::parse();

    if !matches!(cli.command, Commands::Serve { .. } | Commands::Browse { .. } | Commands::Init) {
        pretty_env_logger::init();
    }

    if let Commands::Init = cli.command {
        modes::init::initialize_project().await?;
        return Ok(());
    }

    log::info!("Application starting...");

    // shared setup
    log::info!("Building agent...");
    let (agent, model, provider, model_name) = agent::build_agent().await?;
    log::info!("Agent built successfully.");
    // session
    let sessions = SqliteSessionService::new("sessions.db?mode=rwc").await?;
    sessions.migrate().await?;
    let sessions = Arc::new(sessions);
    // memory
    let memory = SqliteMemoryService::new("sqlite:memory.db?mode=rwc").await?;
    memory.migrate().await?;
    let memory = Arc::new(memory);
    let _ = crate::tools::memory::MEMORY_SVC.set(memory.clone());
    let memory_adapter: Arc<dyn adk_rust::Memory> = Arc::new(
        adk_memory::MemoryServiceAdapter::new(memory.clone(), "nami", "default_user"),
    );

    match cli.command {
        Commands::Bot => {
            log::info!("Running in bot mode");
            let runner = Arc::new(AgentRunner::new(
                agent,
                sessions.clone(),
                memory_adapter,
                "telegram",
                model,
            ));
            modes::bot::run_bot(runner, sessions.clone()).await?;
        }
        Commands::Cli => {
            log::info!("Running in CLI mode");
            modes::cli::run_cli(agent, sessions, model, provider, model_name).await?;
        }
        Commands::Run { prompt } => {
            log::info!("Running in direct run mode");
            modes::run::run_direct(agent, &prompt).await?;
        }
        Commands::Serve { port } => {
            log::info!("Running in serve mode");
            modes::serve::run_serve(agent, model, memory_adapter, port.unwrap_or(8080)).await?;
        }
        Commands::Browse { port } => {
            log::info!("Running in browse mode");
            modes::browse::run_browse(agent, model, memory_adapter, port.unwrap_or(8080)).await?;
        }
        Commands::Init => unreachable!(),
    }

    // shutdown telemetry
    if !otel_endpoint.is_empty() {
        shutdown_telemetry();
    }
    Ok(())
}
