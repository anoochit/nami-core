pub async fn read_file_lines(path: &std::path::Path, max_lines: Option<usize>) -> std::result::Result<String, String> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if let Some(limit) = max_lines {
        let lines: Vec<&str> = content.lines().take(limit).collect();
        Ok(lines.join("\n"))
    } else {
        Ok(content)
    }
}