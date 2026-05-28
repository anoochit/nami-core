// Basic sanity unit test for search tool
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_google_search_missing_key() {
        let args = SearchArgs { query: "test".to_string() };
        let result = google_search(args).await;
        // Expect an error due to missing SERPER_API_KEY env var
        assert!(result.is_err());
    }
}
