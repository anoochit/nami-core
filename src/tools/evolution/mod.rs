use crate::utils::{get_nami_dir, get_workspace_dir};
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_rust::prelude::*;
use adk_tool::AdkError;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::fs;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use futures::StreamExt;
use std::process::Command;
use std::path::Path;

#[derive(Deserialize)]
pub struct ProposeEvolutionArgs {
    pub focus: Option<String>,
}

#[derive(Deserialize)]
pub struct ApplyEvolutionArgs {
    pub select_categories: Option<Vec<String>>,
}

pub struct ProposeEvolution {
    model: Arc<dyn Llm>,
}

impl ProposeEvolution {
    pub fn new(model: Arc<dyn Llm>) -> Self {
        Self { model }
    }
}

#[async_trait::async_trait]
impl Tool for ProposeEvolution {
    fn name(&self) -> &str {
        "propose_evolution"
    }

    fn description(&self) -> &str {
        "Scans recent interaction logs and errors, then generates an evolutionary proposal (workspace/EVOLUTION_PLAN.md) for updating memories, agent behavior, or repairing tools."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "focus": { "type": "string", "description": "Optional custom focus or direction for this evolutionary cycle." }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn adk_rust::tool::ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: ProposeEvolutionArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        let nami_dir = get_nami_dir();
        let db_path = nami_dir.join("sessions.db");
        
        if !db_path.exists() {
            return Err(AdkError::tool("No active sessions.db database found to analyze. Please chat more first!"));
        }

        let db_url = format!("sqlite:{}?mode=ro", db_path.to_string_lossy());
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to connect to sessions.db: {}", e)))?;

        // 1. Fetch recent events
        let rows = sqlx::query(
            "SELECT author, timestamp, llm_response FROM events \
             WHERE author != 'reflection_service' \
             ORDER BY timestamp DESC LIMIT 150"
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to fetch logs: {}", e)))?;

        let mut conversation_snippet = String::new();
        for row in rows.iter().rev() {
            let author: String = row.get(0);
            let llm_json: String = row.get(2);
            let role = if author == "nami" { "Nami" } else { "User" };
            let text = extract_text_from_json(&llm_json);
            if !text.is_empty() {
                conversation_snippet.push_str(&format!("{}: {}\n", role, text));
            }
        }

        // 2. Read current memories and AGENT.md
        let workspace_dir = get_workspace_dir().await.map_err(|e| AdkError::tool(e.to_string()))?;
        let memories_path = workspace_dir.join("MEMORIES.md");
        let agent_path = workspace_dir.join("AGENT.md");

        let current_memories = fs::read_to_string(&memories_path).await.unwrap_or_default();
        let current_agent_rules = fs::read_to_string(&agent_path).await.unwrap_or_default();

        let focus_instruction = args.focus.as_ref().map_or("".to_string(), |f| format!("\nSpecial Evolutionary Focus: {}\n", f));

        // 3. Ask LLM to propose evolution plan
        let prompt = format!(
            r#"You are Nami's Core Evolution Engine. Analyze the recent conversation snippet below to identify:
1. New personal details or explicit user preferences to add/update in MEMORIES.md.
2. Behavioral patterns, styling preferences, or workflow adjustments to update in Nami's AGENT.md rules.
3. Code refinements or tool improvements if any repetitive errors or compile warnings were mentioned.

Existing Memories (MEMORIES.md):
{}

Existing Agent Rules (AGENT.md):
{}
{}
Recent Conversation History:
{}

Based on the above, draft a precise evolution proposal. Format your entire output as a valid markdown file that will be saved as EVOLUTION_PLAN.md.
Include sections:
- ## 1. Summary of Recent Insights
- ## 2. Proposed MEMORIES.md Updates (using + / - diff blocks or bullet lists of what to add/edit)
- ## 3. Proposed AGENT.md Heuristic Diffs (precise rule modifications in standard unified diff format)
- ## 4. Proposed Skill/Code Fixes (if any)

Be extremely specific and practical. Return only the markdown content, no wrapping fences or comments."#,
            current_memories, current_agent_rules, focus_instruction, conversation_snippet
        );

        let mut stream = self.model.generate_content(
            LlmRequest::new(
                "gemini-2.5-pro".to_string(), // use top-tier model for evolution planning
                vec![Content::new("user").with_text(&prompt)],
            ),
            false,
        ).await.map_err(|e| AdkError::tool(format!("LLM generation failed: {}", e)))?;

        let mut plan_content = String::new();
        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| AdkError::tool(e.to_string()))?;
            if let Some(content) = event.content {
                for part in content.parts {
                    if let Some(t) = part.text() {
                        plan_content.push_str(t);
                    }
                }
            }
        }

        // 4. Save proposal
        let plan_path = workspace_dir.join("EVOLUTION_PLAN.md");
        fs::write(&plan_path, &plan_content).await.map_err(|e| AdkError::tool(format!("Failed to write EVOLUTION_PLAN.md: {}", e)))?;

        Ok(json!({
            "status": "success",
            "message": "Evolution proposal generated successfully.",
            "path": "workspace/EVOLUTION_PLAN.md",
            "preview": plan_content.chars().take(800).collect::<String>() + "..."
        }))
    }
}

pub struct ApplyEvolution;

#[async_trait::async_trait]
impl Tool for ApplyEvolution {
    fn name(&self) -> &str {
        "apply_evolution"
    }

    fn description(&self) -> &str {
        "Applies the approved updates proposed in workspace/EVOLUTION_PLAN.md to MEMORIES.md, AGENT.md, or skill tools, and creates a Git commit."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "select_categories": { "type": "array", "items": { "type": "string" }, "description": "Optional: Only apply specific updates. If None, applies everything." }
            }
        }))
    }

    async fn execute(&self, _ctx: Arc<dyn adk_rust::tool::ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let _args: ApplyEvolutionArgs = serde_json::from_value(args).map_err(|e| AdkError::tool(e.to_string()))?;
        let workspace_dir = get_workspace_dir().await.map_err(|e| AdkError::tool(e.to_string()))?;
        let plan_path = workspace_dir.join("EVOLUTION_PLAN.md");

        if !plan_path.exists() {
            return Err(AdkError::tool("No proposed EVOLUTION_PLAN.md found in workspace to apply. Run propose_evolution first!"));
        }

        let plan_content = fs::read_to_string(&plan_path).await.map_err(|e| AdkError::tool(e.to_string()))?;

        // Parse and apply changes to MEMORIES.md or AGENT.md
        let memories_path = workspace_dir.join("MEMORIES.md");
        let agent_path = workspace_dir.join("AGENT.md");

        let mut memories_updated = false;
        let mut agent_updated = false;

        if plan_content.contains("MEMORIES.md") {
            if let Some(mem_block) = extract_block(&plan_content, "MEMORIES.md") {
                fs::write(&memories_path, mem_block).await.map_err(|e| AdkError::tool(e.to_string()))?;
                memories_updated = true;
            }
        }
        if plan_content.contains("AGENT.md") {
            if let Some(agent_block) = extract_block(&plan_content, "AGENT.md") {
                fs::write(&agent_path, agent_block).await.map_err(|e| AdkError::tool(e.to_string()))?;
                agent_updated = true;
            }
        }

        // Clean up the plan now that it's applied
        let _ = fs::remove_file(&plan_path).await;

        // Git Commit
        let git_status = if Path::new(".git").exists() {
            let mut action_msg = "evolution: self-update".to_string();
            if memories_updated { action_msg.push_str(" memories"); }
            if agent_updated { action_msg.push_str(" and agent heuristics"); }

            let _ = Command::new("git").args(&["add", "workspace/MEMORIES.md", "workspace/AGENT.md"]).output();
            let commit_res = Command::new("git").args(&["commit", "-m", &action_msg]).output();
            match commit_res {
                Ok(out) => format!("Changes committed to Git: {}", String::from_utf8_lossy(&out.stdout).trim()),
                Err(e) => format!("Failed to commit to Git: {}", e),
            }
        } else {
            "Git is not initialized, skipping commit.".to_string()
        };

        Ok(json!({
            "status": "success",
            "message": "Evolution plan applied successfully.",
            "memories_updated": memories_updated,
            "agent_updated": agent_updated,
            "git_status": git_status
        }))
    }
}

pub fn evolution_tools(model: Arc<dyn Llm>) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(ProposeEvolution::new(model)),
        Arc::new(ApplyEvolution),
    ]
}

// Helpers
fn extract_text_from_json(llm_json: &str) -> String {
    let Ok(val) = serde_json::from_str::<Value>(llm_json) else {
        return String::new();
    };

    if let Some(parts) = val.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
        return parts.iter()
            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
    }
    if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
        return text.to_string();
    }
    if let Some(text) = val.get("content").and_then(|c| c.as_str()) {
        return text.to_string();
    }
    String::new()
}

fn extract_block(full_text: &str, file_keyword: &str) -> Option<String> {
    let mut current_section = "";
    let mut block_lines = Vec::new();
    let mut in_block = false;

    for line in full_text.lines() {
        if line.starts_with("##") {
            current_section = line;
        }
        if current_section.contains(file_keyword) {
            if line.starts_with("```") {
                if in_block {
                    return Some(block_lines.join("\n"));
                } else {
                    in_block = true;
                    continue;
                }
            }
            if in_block {
                block_lines.push(line);
            }
        }
    }
    None
}
