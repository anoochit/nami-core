use std::sync::Arc;
use adk_memory::{MemoryService, MemoryServiceAdapter, SqliteMemoryService};
use adk_session::{SessionService, SqliteSessionService};
use anyhow::Result;

use crate::utils::get_nami_dir;

pub struct Dependencies {
    pub sessions: Arc<dyn SessionService>,
    pub memory: Arc<dyn MemoryService>,
    pub memory_adapter: Arc<dyn adk_rust::Memory>,
}

pub async fn setup_dependencies() -> Result<Dependencies> {
    let nami_dir = get_nami_dir();
    let sessions_db = nami_dir.join("sessions.db");
    let sessions_url = format!("sqlite:{}?mode=rwc", sessions_db.to_string_lossy());

    let memory_db = nami_dir.join("memory.db");
    let memory_url = format!("sqlite:{}?mode=rwc", memory_db.to_string_lossy());

    // Initialize Sessions Service (SQLite only)
    let sessions: Arc<dyn SessionService> = {
        let svc = SqliteSessionService::new(&sessions_url).await?;
        svc.migrate().await?;
        Arc::new(svc)
    };

    // Initialize Memory Service (SQLite only)
    let memory: Arc<dyn MemoryService> = {
        let svc = SqliteMemoryService::new(&memory_url).await?;
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
