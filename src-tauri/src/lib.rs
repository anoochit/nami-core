use nami::agent;
use nami::modes::startup::setup_dependencies;
use nami::modes::serve::run_serve;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Ensure we are in the project root so config.toml and DBs can be found
  let _ = std::env::set_current_dir("..");

  tauri::Builder::default()
    .setup(|_app| {
      // Start Nami API server in a dedicated thread to avoid lifetime/runtime issues
      std::thread::spawn(|| {
        println!(">>> Starting Nami Backend Thread...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
          println!(">>> Nami Server Runtime Started. Initializing...");
          if let Err(e) = start_nami_server().await {
            eprintln!("!!! CRITICAL: Nami Server failed to start: {:?}", e);
          }
        });
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

async fn start_nami_server() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    println!(">>> Building agent...");
    let (agent, model, _provider, _model_name) = agent::build_agent().await?;
    println!(">>> Setting up dependencies...");
    let deps = setup_dependencies().await?;

    println!(">>> Starting server on 127.0.0.1:8080...");
    let _ = run_serve(
        agent,
        model,
        deps.sessions,
        deps.memory_adapter,
        "127.0.0.1".to_string(),
        8080,
    ).await;
    Ok(())
}
