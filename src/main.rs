mod agent;
mod modes;
mod runner;
mod tools;
mod utils;

use std::sync::Arc;

use adk_session::SqliteSessionService;
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
    Hud { session_id: Option<String> }, // dashboard
    Init,                        // initialize project files
    Run { prompt: String },      // direct execution
    Serve { port: Option<u16> }, // http server
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if !matches!(cli.command, Commands::Serve { .. } | Commands::Init) {
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
    let sessions = SqliteSessionService::new("sessions.db?mode=rwc").await?;
    sessions.migrate().await?;
    let sessions = Arc::new(sessions);

    match cli.command {
        Commands::Bot => {
            log::info!("Running in bot mode");
            let runner = Arc::new(AgentRunner::new(agent, sessions.clone(), "telegram", model));
            modes::bot::run_bot(runner, sessions.clone()).await?;
        }
        Commands::Cli => {
            log::info!("Running in CLI mode");
            modes::cli::run_cli(agent, sessions, model, provider, model_name).await?;
        }
        Commands::Hud { session_id } => {
            log::info!("Running in HUD mode");
            modes::hud::run_hud(agent, sessions, model, provider, model_name, session_id).await?;
        }
        Commands::Run { prompt } => {
            log::info!("Running in direct run mode");
            modes::run::run_direct(agent, &prompt).await?;
        }
        Commands::Serve { port } => {
            log::info!("Running in serve mode");
            modes::serve::run_serve(agent, model, port.unwrap_or(8080)).await?;
        }
        Commands::Init => unreachable!(),
    }

    Ok(())
}