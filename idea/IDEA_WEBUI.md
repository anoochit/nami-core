# Idea: Overriding adk-server WebUI without library modification

To replace the WebUI without touching the `adk-server` crate or recompiling it when assets change, we can use an Axum middleware hijack in the main application (`namiclaw`).

## Approach: Middleware Hijack

Since `adk-server` uses Axum, we can wrap the generated `Router` with a layer that intercepts requests to `/ui/` and serves them from a local filesystem directory instead of the embedded assets.

### 1. Requirements
Add `tower-http` to `Cargo.toml`:
```toml
tower-http = { version = "0.6", features = ["fs"] }
```

### 2. Implementation Pattern
In `src/modes/serve.rs` (or wherever the server is started):

```rust
use axum::{middleware, Router, response::IntoResponse};
use tower_http::services::ServeDir;
use tower::ServiceExt;

pub(crate) async fn run_serve(
    agent: Arc<dyn Agent>,
    model: Arc<dyn Llm>,
    port: u16,
) -> anyhow::Result<()> {
    let base_url = std::env::var("A2A_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // 1. Build the app using the launcher
    let launcher = Launcher::new(agent)
        .with_compaction(get_compaction_config(model))
        .with_a2a_base_url(base_url);

    let app = launcher.build_app()?;

    // 2. Define the local directory for the new UI
    // This can be configurable via environment variable
    let ui_dir = std::env::var("CUSTOM_UI_DIR").unwrap_or_else(|_| "./custom-webui".to_string());
    let new_ui_service = ServeDir::new(ui_dir);

    // 3. Apply the hijack layer
    let app = app.layer(middleware::from_fn(move |req, next| {
        let path = req.uri().path().to_string();
        let service = new_ui_service.clone();
        
        async move {
            if path.starts_with("/ui/") {
                // Serve from the local folder
                // Note: ServeDir expects the path to match the directory structure
                // We might need to strip the "/ui/" prefix if the folder doesn't have it
                service.oneshot(req).await.unwrap().into_response()
            } else {
                // Pass through to the adk-server handlers
                next.run(req).await
            }
        }
    }));

    // 4. Start the server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

## Benefits
- **No modification to `ref/adk-server`**: Keep the reference library clean.
- **Dynamic Updates**: Swap UI files without rebuilding the Rust project.
- **Flexibility**: Easily toggle between the embedded UI and a custom one using environment variables.
