// src/tools/soul/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn test_update_user_memory_success() {
        // Set a temporary workspace directory
        env::set_var("WORKSPACE_DIR", "./temp_workspace");
        // Ensure the directory exists
        let _ = tokio::fs::create_dir_all("./temp_workspace").await;
        let args = UpdateMemoryArgs { fact: "test fact".to_string() };
        let result = update_user_memory(args).await;
        // Should succeed and create MEMORIES.md
        assert!(result.is_ok());
    }
}
