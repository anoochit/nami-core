use adk_memory::{MemoryEntry, MemoryService, SearchRequest};
#[cfg(test)]
use adk_memory::SqliteMemoryService;
use adk_rust::Content;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};

pub static MEMORY_SVC: OnceLock<Arc<dyn MemoryService>> = OnceLock::new();

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

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn rank_memories(query: &str, memories: &mut Vec<Value>) {
    if memories.is_empty() {
        return;
    }

    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        return;
    }

    let n_docs = memories.len();
    let docs_tokens: Vec<Vec<String>> = memories
        .iter()
        .map(|m| {
            let text = m["text"].as_str().unwrap_or("");
            tokenize(text)
        })
        .collect();

    // Calculate document frequencies for query terms
    let mut df = std::collections::HashMap::new();
    for token in &query_tokens {
        let count = docs_tokens
            .iter()
            .filter(|doc| doc.contains(token))
            .count();
        df.insert(token.clone(), count);
    }

    // Calculate scores
    let mut scored_memories: Vec<(f64, Value)> = memories
        .drain(..)
        .enumerate()
        .map(|(i, memory)| {
            let doc = &docs_tokens[i];
            let mut score = 0.0;
            if !doc.is_empty() {
                for token in &query_tokens {
                    let tf = doc.iter().filter(|t| *t == token).count() as f64 / doc.len() as f64;
                    let doc_freq = *df.get(token).unwrap_or(&0);
                    let idf = ((1.0 + n_docs as f64) / (1.0 + doc_freq as f64)).ln();
                    score += tf * idf;
                }
            }
            (score, memory)
        })
        .collect();

    // Sort by score descending (using stable sort to preserve order for equal scores)
    scored_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Put them back into memories vector
    for (_, memory) in scored_memories {
        memories.push(memory);
    }
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
            let mut memories: Vec<_> = resp
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
            
            rank_memories(&args.query, &mut memories);

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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to initialize memory for tests
    async fn init_test_memory() {
        let db_path = "test_memory.db";
        // Attempt to delete old DB to ensure clean state
        let _ = tokio::fs::remove_file(db_path).await;
        
        let svc = SqliteMemoryService::new(db_path).await.expect("Failed to create memory service");
        // Attempt to migrate, which should create the tables
        svc.migrate().await.expect("Failed to migrate memory database");
        let _ = MEMORY_SVC.set(Arc::new(svc));
    }

    #[tokio::test]
    async fn test_memory_lifecycle() {
        init_test_memory().await;
        
        // Add a memory
        let add_args = AddMemoryArgs {
            text: "The user prefers to work in a dark theme.".to_string(),
        };
        let add_result = add_memory(add_args).await.unwrap();
        assert_eq!(add_result["status"], "success");

        // Recall the memory
        let recall_args = RecallArgs {
            query: "theme".to_string(),
        };
        let recall_result = recall_memory(recall_args).await.unwrap();
        
        assert_eq!(recall_result["found"], 1);
        assert!(recall_result["memories"][0]["text"].as_str().unwrap().contains("dark theme"));
    }

    #[test]
    fn test_memory_tfidf_ranking() {
        let mut memories = vec![
            json!({"author": "user", "text": "This is completely irrelevant context about apples.", "timestamp": "2026-07-06T11:00:00Z"}),
            json!({"author": "user", "text": "The theme is dark theme, which the user prefers.", "timestamp": "2026-07-06T11:01:00Z"}),
            json!({"author": "user", "text": "A light theme is also an option, but not preferred.", "timestamp": "2026-07-06T11:02:00Z"}),
        ];

        rank_memories("dark theme preferred", &mut memories);

        assert!(memories[0]["text"].as_str().unwrap().contains("dark theme"));
        assert!(memories[1]["text"].as_str().unwrap().contains("light theme"));
        assert!(memories[2]["text"].as_str().unwrap().contains("apples"));
    }
}
