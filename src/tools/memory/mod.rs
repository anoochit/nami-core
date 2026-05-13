use adk_memory::{MemoryEntry, MemoryService, SearchRequest, SqliteMemoryService};
use adk_rust::Content;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};

pub static MEMORY_SVC: OnceLock<Arc<SqliteMemoryService>> = OnceLock::new();

#[derive(Deserialize, JsonSchema)]
pub struct RecallArgs {
    /// What to search for in memory
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddMemoryArgs {
    /// The text or fact to remember
    pub text: String,
}

/// Search long-term memory for relevant past conversations or facts.
#[tool]
async fn recall_memory(args: RecallArgs) -> std::result::Result<Value, AdkError> {
    let svc = MEMORY_SVC.get().ok_or_else(|| AdkError::tool("Memory service not initialized"))?;
    
    match svc
        .search(SearchRequest {
            query: args.query.clone(),
            user_id: "default_user".into(),
            app_name: "nami".into(),
            limit: Some(5),
            min_score: None,
            project_id: None,
        })
        .await
    {
        Ok(resp) => {
            let memories: Vec<_> = resp
                .memories
                .iter()
                .map(|m| {
                    let text: String = m.content.parts.iter().filter_map(|p| p.text()).collect();
                    json!({
                        "author": m.author,
                        "text": text,
                        "timestamp": m.timestamp,
                    })
                })
                .collect();
            Ok(json!({
                "query": args.query,
                "found": memories.len(),
                "memories": memories,
            }))
        }
        Err(e) => Err(AdkError::tool(format!("Memory search failed: {}", e))),
    }
}

/// Explicitly save a new fact or context to long-term memory.
#[tool]
async fn add_memory(args: AddMemoryArgs) -> std::result::Result<Value, AdkError> {
    let svc = MEMORY_SVC.get().ok_or_else(|| AdkError::tool("Memory service not initialized"))?;
    
    let entry = MemoryEntry {
        content: Content::new("user").with_text(&args.text),
        author: "user".into(),
        timestamp: chrono::Utc::now(),
    };

    match svc.add_session("nami", "default_user", "explicit_memory", vec![entry]).await {
        Ok(_) => Ok(json!({"status": "success", "message": "Memory saved."})),
        Err(e) => Err(AdkError::tool(format!("Failed to save memory: {}", e))),
    }
}

pub fn memory_tools() -> Vec<Arc<dyn adk_rust::Tool>> {
    vec![Arc::new(RecallMemory), Arc::new(AddMemory)]
}
