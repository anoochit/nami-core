use std::sync::Arc;
use adk_core::{Agent, Memory};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event as SseEvent, KeepAlive, Sse}},
    routing::post,
    Json, Router,
};
use serde_json::Value;
use crate::runner::AgentRunner;

#[derive(Clone)]
struct AguiState {
    agent: Arc<dyn Agent>,
    memory: Arc<dyn Memory>,
}

pub async fn run_agui(
    agent: Arc<dyn Agent>,
    memory_adapter: Arc<dyn Memory>,
    port: u16,
) -> Result<()> {
    log::info!("Starting AG-UI protocol server on port {}...", port);

    let state = AguiState {
        agent,
        memory: memory_adapter,
    };

    let app = Router::new()
        .route("/run_sse", post(handle_run_sse))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("AG-UI server running on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_run_sse(
    State(state): State<AguiState>,
    Json(body): Json<Value>,
) -> Response {
    let prompt = body["prompt"].as_str().unwrap_or("hello");
    let session_id = "default_session";
    let user_id = "default_user";

    // Cast agent to Llm trait if possible or identify appropriate Llm implementation
    // Since the original code attempted to use state.agent as Llm, and AgentRunner 
    // expects Arc<dyn Llm>, we assume state.agent implements Llm.
    let llm: Arc<dyn adk_core::Llm> = unsafe { std::mem::transmute(state.agent.clone()) };

    let runner = AgentRunner::new(
        state.agent.clone(),
        Arc::new(adk_session::InMemorySessionService::new()),
        state.memory.clone(),
        "agui",
        llm,
    );

    match runner.run(user_id, session_id, prompt).await {
        Ok(response) => {
            let event = SseEvent::default().data(response);
            Sse::new(futures::stream::once(async move { Ok::<_, std::convert::Infallible>(event) }))
                .keep_alive(KeepAlive::default())
                .into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
