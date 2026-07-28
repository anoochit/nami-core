use crate::utils::sandbox;
use adk_rust::prelude::*;
use adk_tool::tool;
use serde_json::{Value, json};
use tokio::fs;
use futures::future::try_join_all;
use super::{WriteFileArgs, ReplaceArgs, MergeFilesArgs, PathArgs};

#[tool]
pub(crate) async fn write_file(args: WriteFileArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::write(&path, &args.content)
        .await
        .map_err(|e| AdkError::tool(format!("Write failed: {}", e)))?;

    Ok(json!({ "status": "success", "path": args.path }))
}

#[tool]
pub(crate) async fn delete_file(args: PathArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    if !path.exists() {
        return Err(AdkError::tool(format!("File does not exist: {}", args.path)));
    }
    if !path.is_file() {
        return Err(AdkError::tool(format!("Path is not a file: {}", args.path)));
    }

    fs::remove_file(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Delete failed: {}", e)))?;

    Ok(json!({ "status": "success" }))
}

#[tool]
pub(crate) async fn replace_text(args: ReplaceArgs) -> std::result::Result<Value, AdkError> {
    let path = sandbox(&args.path).await?;
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Read failed: {}", e)))?;

    let lines: Vec<&str> = content.split('\n').collect();
    let total_lines = lines.len();

    let start_line = args.start_line.unwrap_or(1);
    if start_line == 0 {
        return Err(AdkError::tool("start_line must be 1-indexed (greater than 0)".to_string()));
    }

    let start_idx = (start_line - 1).min(total_lines);
    let end_idx = match args.end_line {
        Some(e) => e.min(total_lines),
        None => total_lines,
    };

    if start_idx > end_idx {
        return Err(AdkError::tool("start_line cannot be greater than end_line".to_string()));
    }

    let pre_lines = &lines[0..start_idx];
    let target_lines = &lines[start_idx..end_idx];
    let post_lines = &lines[end_idx..];

    let target_block = target_lines.join("\n");

    let matches: Vec<_> = target_block.matches(&args.old_string).collect();
    let count = matches.len();

    if count == 0 {
        return Err(AdkError::tool("old_string not found in the specified line range".to_string()));
    }

    let allow_multiple = args.allow_multiple.unwrap_or(false);
    if count > 1 && !allow_multiple {
        return Err(AdkError::tool(format!(
            "Found {} occurrences of old_string in the specified line range, but allow_multiple is false",
            count
        )));
    }

    let new_target_block = if allow_multiple {
        target_block.replace(&args.old_string, &args.new_string)
    } else {
        target_block.replacen(&args.old_string, &args.new_string, 1)
    };

    let mut final_lines = Vec::new();
    final_lines.extend(pre_lines.iter().copied());
    final_lines.push(&new_target_block);
    final_lines.extend(post_lines.iter().copied());

    let final_content = final_lines.join("\n");

    fs::write(&path, final_content)
        .await
        .map_err(|e| AdkError::tool(format!("Write failed: {}", e)))?;

    Ok(json!({ "status": "success", "replaced_occurrences": count }))
}

#[tool]
pub(crate) async fn merge_files(args: MergeFilesArgs) -> std::result::Result<Value, AdkError> {
    let separator = args.separator.unwrap_or_else(|| "\n".to_string());

    let mut resolved_paths = Vec::new();
    for file_path in &args.input_files {
        let path = sandbox(file_path).await?;
        resolved_paths.push((file_path.clone(), path));
    }

    let read_futures = resolved_paths.into_iter().map(|(file_path, path)| async move {
        fs::read_to_string(&path)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to read {}: {}", file_path, e)))
    });

    let contents = try_join_all(read_futures).await?;
    let combined_content = contents.join(&separator);

    let out_path = sandbox(&args.output_file).await?;

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::write(&out_path, combined_content).await.map_err(|e| {
        AdkError::tool(format!(
            "Failed to write merged output to {}: {}",
            args.output_file, e
        ))
    })?;

    Ok(
        json!({ "status": "success", "message": format!("Merged {} files into {}", args.input_files.len(), args.output_file) }),
    )
}
