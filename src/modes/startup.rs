use std::sync::Arc;
use adk_memory::{MemoryService, MemoryServiceAdapter, SqliteMemoryService};
use adk_session::{SessionService, SqliteSessionService};
use anyhow::Result;

pub struct Dependencies {
    pub sessions: Arc<dyn SessionService>,
    pub memory: Arc<dyn MemoryService>,
    pub memory_adapter: Arc<dyn adk_rust::Memory>,
}

pub async fn setup_dependencies() -> Result<Dependencies> {


    // Initialize Sessions Service (SQLite only)
    let sessions: Arc<dyn SessionService> = {
        let svc = SqliteSessionService::new("sessions.db?mode=rwc").await?;
        svc.migrate().await?;
        Arc::new(svc)
    };

    // Initialize Memory Service (SQLite only)
    let memory: Arc<dyn MemoryService> = {
        let svc = SqliteMemoryService::new("sqlite:memory.db?mode=rwc").await?;
        svc.migrate().await?;
        Arc::new(svc)
    };

    // Set global memory service reference
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
