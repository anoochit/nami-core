use adk_telemetry::{init_with_otlp, shutdown_telemetry};
use clap::{Parser, Subcommand};
use nami::agent;
use nami::modes;
use nami::modes::init::run_init;
use nami::modes::startup::setup_dependencies;
use nami::runner::AgentRunner;
use tracing_subscriber::{fmt, util::SubscriberInitExt};
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

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
        prompt: String,
    },
    /// Start the HTTP server.
    Serve {
        /// Optional port to serve on.
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
    },
    /// Manage registered workspaces.
    Workspace {
        #[command(subcommand)]
        subcommand: WorkspaceCommands,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Add a workspace directory by path.
    Add {
        /// Path to the workspace directory.
        path: String,
    },
    /// List all registered workspaces.
    List,
    /// Select a workspace as active.
    Select {
        /// Index (1-based) or path of the workspace to select.
        index_or_path: String,
    },
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let is_quiet = matches!(
        cli.command,
        Commands::Run { .. } | Commands::Eval | Commands::Workspace { .. }
    );

    if !is_quiet {
        println!("Application entry reached");
    }
    log::info!("Application entry reached");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().ok();

    if !is_quiet {
        println!("Parsing CLI...");
    }

    // Logging & Telemetry setup
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "".to_string());
    if rust_log.is_empty() {
        if is_quiet {
            unsafe { std::env::set_var("RUST_LOG", "error") };
        } else {
            unsafe { std::env::set_var("RUST_LOG", "info") };
        }
    } else if is_quiet {
        unsafe { std::env::set_var("RUST_LOG", "error") };
    }
    if is_quiet {
        unsafe { std::env::set_var("npm_config_loglevel", "error") };
        unsafe { std::env::set_var("NPM_CONFIG_LOGLEVEL", "error") };
    }

    if !is_quiet {
        println!(
            "RUST_LOG set to '{}'",
            std::env::var("RUST_LOG").unwrap()
        );
    }

    let otel_endpoint = std::env::var("OTEL_COLLECTOR").unwrap_or_else(|_| "NOT_SET".to_string());
    let use_telemetry = otel_endpoint != "NOT_SET" && !otel_endpoint.is_empty();
    if !is_quiet {
        println!("OTEL_COLLECTOR='{}'", otel_endpoint);
        println!("Initializing telemetry...");
    }

    // If we are running under Tauri, let Tauri's plugin manage logging
    let is_tauri = std::env::var("TAURI_ENV").is_ok();

    if !is_tauri {
        // If in CLI/TUI mode, log to file, otherwise stdout
        let is_interactive = matches!(cli.command, Commands::Cli | Commands::Tui);

        if is_interactive {
            let log_path = nami::utils::get_nami_dir().join("nami.log");
            let log_file = File::create(log_path).expect("Failed to create log file");
            let _ = tracing_subscriber::registry()
                .with(fmt::layer().with_writer(log_file))
                .try_init();
        } else if use_telemetry {
            init_with_otlp("nami", &otel_endpoint).expect("Failed to initialize telemetry");
        }
    }

    if !is_quiet {
        println!("Telemetry initialized. Starting app...");
    }
    log::info!("Application starting with telemetry: {}", use_telemetry);
    tracing::info!("Application starting with telemetry: {}", use_telemetry);

    // shared setup
    if !is_quiet {
        log::info!("Building agent...");
    }
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
            workspaces: None,
        }
    });

    let (agent, model, provider, model_name, mcp_count, skill_count) = agent::build_agent().await?;
    log::info!(
        "Agent built successfully with {} MCP servers and {} skills.",
        mcp_count,
        skill_count
    );

    let deps = setup_dependencies().await?;

    // Spawn scheduler background loop
    let agent_scheduler = agent.clone();
    let sessions_scheduler = deps.sessions.clone();
    let model_scheduler = model.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::modes::scheduler::run_scheduler_loop_with_deps(
            agent_scheduler,
            model_scheduler,
            sessions_scheduler,
        )
        .await
        {
            log::error!("Scheduler background error: {:?}", e);
        }
    });

    // Reflection Service
    if config
        .reflection
        .as_ref()
        .map(|r| r.enabled)
        .unwrap_or(false)
    {
        let reflection_model_cfg = config.reflection.as_ref().and_then(|r| r.to_model_config());
        let reflection_model =
            agent::load_model_with_fallback(&reflection_model_cfg, &config.model).await?;
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
            modes::cli::run_cli(agent, deps.sessions, model, provider, model_name, mcp_count, skill_count).await?;
        }
        Commands::Tui => {
            unsafe { std::env::set_var("RUST_LOG", "error") };
            modes::tui::run_tui(
                agent,
                deps.sessions,
                deps.memory_adapter,
                model,
                provider,
                model_name,
                mcp_count,
                skill_count,
            )
            .await?;
        }
        Commands::Run { prompt } => {
            log::info!("Running in direct run mode");
            modes::run::run_direct(agent, &prompt).await?;
        }
        Commands::Serve { port, host } => {
            log::info!("Running in serve mode");
            let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
            modes::serve::run_serve(
                agent,
                model,
                deps.sessions.clone(),
                deps.memory_adapter,
                host,
                port.unwrap_or(8080),
            )
            .await?;
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
        }
        Commands::Eval => {
            log::info!("Running evaluation mode");
            modes::eval::run_eval(agent, deps.sessions, deps.memory_adapter, model).await?;
        }
        Commands::Workspace { subcommand } => {
            log::info!("Running workspace command");
            handle_workspace(subcommand).await?;
        }
    }

    // shutdown telemetry
    if use_telemetry {
        if !is_quiet {
            log::info!("Flushing telemetry...");
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        shutdown_telemetry();
    }
    Ok(())
}

async fn handle_workspace(subcommand: WorkspaceCommands) -> anyhow::Result<()> {
    let mut config = match agent::load_config_sync() {
        Ok(c) => c,
        Err(e) => {
            println!("Error: Failed to load config.toml (please run 'nami init' first). Detail: {}", e);
            return Ok(());
        }
    };

    let mut workspaces = config.workspaces.clone().unwrap_or_default();
    let mut list = workspaces.list.clone().unwrap_or_default();

    match subcommand {
        WorkspaceCommands::Add { path } => {
            let path_buf = std::path::PathBuf::from(&path);
            let absolute_path = std::fs::canonicalize(&path_buf)
                .unwrap_or_else(|_| path_buf.clone())
                .to_string_lossy()
                .replace('\\', "/");

            if !list.contains(&absolute_path) {
                list.push(absolute_path.clone());
                println!("Workspace added: {}", absolute_path);
            } else {
                println!("Workspace already registered: {}", absolute_path);
            }

            workspaces.list = Some(list);
            config.workspaces = Some(workspaces);
            agent::save_config_sync(&config)?;
            println!("Configuration saved.");
        }
        WorkspaceCommands::List => {
            let active = workspaces.active.as_deref().unwrap_or("None");
            println!("Active workspace: {}", active);
            println!("\nRegistered workspaces:");
            for (idx, ws) in list.iter().enumerate() {
                let marker = if ws == active { "*" } else { " " };
                println!("  [{}] {} {}", idx + 1, marker, ws);
            }
        }
        WorkspaceCommands::Select { index_or_path } => {
            let target_path = if let Ok(idx) = index_or_path.parse::<usize>() {
                if idx > 0 && idx <= list.len() {
                    Some(list[idx - 1].clone())
                } else {
                    println!("Error: Index out of bounds (1 to {}).", list.len());
                    return Ok(());
                }
            } else {
                let path_buf = std::path::PathBuf::from(&index_or_path);
                let absolute_path = std::fs::canonicalize(&path_buf)
                    .unwrap_or_else(|_| path_buf.clone())
                    .to_string_lossy()
                    .replace('\\', "/");
                if list.contains(&absolute_path) {
                    Some(absolute_path)
                } else {
                    println!("Error: Workspace path not registered. Use 'workspace add <path>' first.");
                    None
                }
            };

            if let Some(target) = target_path {
                workspaces.active = Some(target.clone());
                config.workspaces = Some(workspaces);
                agent::save_config_sync(&config)?;
                println!("Selected active workspace: {}", target);
            }
        }
    }

    Ok(())
}

