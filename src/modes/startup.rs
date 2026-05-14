use std::sync::Arc;
use adk_memory::{SqliteMemoryService, MemoryServiceAdapter};
use adk_session::SqliteSessionService;
use adk_telemetry::init_with_otlp;
use anyhow::Result;

pub struct Dependencies {
    pub sessions: Arc<SqliteSessionService>,
    pub memory: Arc<SqliteMemoryService>,
    pub memory_adapter: Arc<dyn adk_rust::Memory>,
}

pub async fn setup_dependencies() -> Result<Dependencies> {
    // Telemetry
    let otel_endpoint = std::env::var("OTEL_COLLECTOR").unwrap_or_default();
    if !otel_endpoint.is_empty() {
        log::info!("Init telemetry...");
        init_with_otlp("agent", &otel_endpoint).expect("Failed to initialize telemetry");
    }

    // Sessions
    let sessions = SqliteSessionService::new("sessions.db?mode=rwc").await?;
    sessions.migrate().await?;
    let sessions = Arc::new(sessions);

    // Memory
    let memory = SqliteMemoryService::new("sqlite:memory.db?mode=rwc").await?;
    memory.migrate().await?;
    let memory = Arc::new(memory);
    let _ = crate::tools::memory::MEMORY_SVC.set(memory.clone());
    
    let memory_adapter: Arc<dyn adk_rust::Memory> = Arc::new(
        MemoryServiceAdapter::new(memory.clone(), "nami", "default_user"),
    );

    Ok(Dependencies {
        sessions,
        memory,
        memory_adapter,
    })
}
