use crate::utils::{get_workspace_dir, sandbox};
use adk_rust::prelude::*;
use adk_tool::tool;
use serde_json::{Value, json};
use tokio::fs;
use super::{PathArgs, GrepArgs, GlobArgs};

#[tool]
pub(crate) async fn list_dir(args: PathArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let mut dir = fs::read_dir(&path)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut entries = Vec::new();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        entries.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(json!({ "entries": entries }))
}

#[tool]
pub(crate) async fn grep_search(args: GrepArgs) -> std::result::Result<Value, AdkError> {
    let search_root = match args.path {
        Some(p) => sandbox(&p).await?,
        None => get_workspace_dir().await?,
    };

    let is_case_insensitive = args.case_insensitive.unwrap_or(false);
    let re = regex::RegexBuilder::new(&args.pattern)
        .case_insensitive(is_case_insensitive)
        .build()
        .map_err(|e| AdkError::tool(format!("Invalid regex pattern: {}", e)))?;

    let mut results = Vec::new();
    let mut total_matches = 0;
    let max_matches = 100;

    for entry in walkdir::WalkDir::new(&search_root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "target" && name != "node_modules" && name != ".gemini" && name != "build" && name != "dist"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if content.contains('\0') {
                    continue;
                }
                
                let rel_path = file_path.strip_prefix(&search_root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                for (idx, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        total_matches += 1;
                        if results.len() < max_matches {
                            results.push(json!({
                                "path": rel_path.clone(),
                                "line": idx + 1,
                                "content": line.to_string(),
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "results": results,
        "total_matches": total_matches,
        "truncated": total_matches > max_matches,
    }))
}

#[tool]
pub(crate) async fn glob_find(args: GlobArgs) -> std::result::Result<Value, AdkError> {
    let search_root = match args.cwd {
        Some(c) => sandbox(&c).await?,
        None => get_workspace_dir().await?,
    };

    let glob = globset::Glob::new(&args.pattern)
        .map_err(|e| AdkError::tool(format!("Invalid glob pattern: {}", e)))?
        .compile_matcher();

    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(&search_root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "target" && name != "node_modules" && name != ".gemini"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(&search_root) {
                if glob.is_match(rel_path) {
                    files.push(rel_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(json!({ "files": files }))
}
