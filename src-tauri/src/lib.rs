use nami::agent;
use nami::modes::startup::setup_dependencies;
use nami::modes::serve::run_serve;
use std::path::{PathBuf};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Install rustls crypto provider
  let _ = rustls::crypto::ring::default_provider()
      .install_default();

  // Try to find the project root dynamically
  if let Some(root) = find_project_root() {
      println!(">>> Found project root at: {:?}", root);
      let _ = std::env::set_current_dir(&root);
  } else {
      eprintln!("!!! WARNING: Could not find project root. Config files may not be loaded correctly.");
  }

  tauri::Builder::default()
    // .plugin(tauri_plugin_log::Builder::new().build())
    .setup(|_app| {
      // Start Nami API server in a dedicated thread to avoid lifetime/runtime issues
      std::thread::spawn(|| {
        log::info!("Starting Nami Backend Thread...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
          log::info!("Nami Server Runtime Started. Initializing...");
          if let Err(e) = start_nami_server().await {
            log::error!("CRITICAL: Nami Server failed to start: {:?}", e);
          }
        });
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("config.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

async fn start_nami_server() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    log::info!("Building agent...");
    let (agent, model, _provider, _model_name, _mcp_count, _skill_count) = agent::build_agent().await?;
    log::info!("Setting up dependencies...");
    let deps = setup_dependencies().await?;

    log::info!("Starting server on 127.0.0.1:8080...");
    run_serve(
        agent,
        model,
        deps.sessions,
        deps.memory_adapter,
        "127.0.0.1".to_string(),
        8080,
    ).await?;
    Ok(())
}
