use nami::agent;
use nami::modes::startup::setup_dependencies;
use nami::modes::serve::run_serve;
use std::path::PathBuf;
use std::net::TcpListener;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_global_shortcut::{Shortcut, Modifiers, Code, GlobalShortcutExt};

struct ApiPort(Mutex<u16>);

#[tauri::command]
fn minimize_window(window: tauri::Window) {
  let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
  if let Ok(maximized) = window.is_maximized() {
    if maximized {
      let _ = window.unmaximize();
    } else {
      let _ = window.maximize();
    }
  }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
  let _ = window.close();
}

#[tauri::command]
fn get_api_port(port: State<'_, ApiPort>) -> u16 {
  *port.0.lock().unwrap()
}

fn find_free_port(start_port: u16) -> u16 {
    let mut port = start_port;
    loop {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
        port += 1;
        if port == 0 {
            return start_port; // wrapped around
        }
    }
}

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

  let port = find_free_port(8080);

  tauri::Builder::default()
// .plugin(tauri_plugin_log::Builder::new().build())
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .plugin(tauri_plugin_notification::init())
    .plugin({

      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app: &tauri::AppHandle,
                         _shortcut: &tauri_plugin_global_shortcut::Shortcut,
                         event: tauri_plugin_global_shortcut::ShortcutEvent| {
          if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            if let Some(window) = app.get_webview_window("main") {
              let is_visible = window.is_visible().unwrap_or(false);
              if is_visible {
                let _ = window.hide();
              } else {
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
          }
        })
        .build()
    })
    .manage(ApiPort(Mutex::new(port)))
    .invoke_handler(tauri::generate_handler![
      minimize_window,
      maximize_window,
      close_window,
      get_api_port
    ])
    .setup(move |app| {
      // Register global shortcut Alt+Space
      let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
      let _ = app.global_shortcut().register(shortcut);

      // System Tray Menu & Setup
      let toggle = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&toggle, &quit])?;

      let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
          match event.id.as_ref() {
            "toggle" => {
              if let Some(window) = app.get_webview_window("main") {
                let show = if let Ok(visible) = window.is_visible() {
                  !visible
                } else {
                  true
                };
                if show {
                  let _ = window.show();
                  let _ = window.set_focus();
                } else {
                  let _ = window.hide();
                }
              }
            }
            "quit" => {
              std::process::exit(0);
            }
            _ => {}
          }
        })
                .on_tray_icon_event(|tray, event| {
            // Handle left click (mouse button up)
            if let tauri::tray::TrayIconEvent::Click { button, button_state, .. } = event {
                if button == tauri::tray::MouseButton::Left && button_state == tauri::tray::MouseButtonState::Up {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

      // Start Nami API server in a dedicated thread
      std::thread::spawn(move || {
        println!("Starting Nami Backend Thread on port {}...", port);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
          println!("Nami Server Runtime Started. Initializing...");
          if let Err(e) = start_nami_server(port).await {
            eprintln!("CRITICAL: Nami Server failed to start: {:?}", e);
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

async fn start_nami_server(port: u16) -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    println!("Building agent...");
    let (agent, model, _provider, _model_name, _mcp_count, _skill_count) = agent::build_agent().await?;
    println!("Setting up dependencies...");
    let deps = setup_dependencies().await?;

    println!("Starting server on 127.0.0.1:{}...", port);
    run_serve(
        agent,
        model,
        deps.sessions,
        deps.memory_adapter,
        "127.0.0.1".to_string(),
        port,
    ).await?;
    Ok(())
}
