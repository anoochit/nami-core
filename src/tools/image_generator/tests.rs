use std::sync::Arc;
use crate::tools::image_generator::image_generator_tools;

#[tokio::test]
async fn test_image_generator_no_api_key() {
    // Ensure credentials and project environment variables are not set
    unsafe {
        std::env::remove_var("GOOGLE_API_KEY");
        std::env::remove_var("GOOGLE_CLOUD_PROJECT");
        std::env::remove_var("GCP_PROJECT");
    }
    let tools = image_generator_tools(None);
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    
    // Call the tool with a dummy prompt, expecting an error about missing credentials and project id
    let args = serde_json::json!({"prompt": "a cat", "aspect_ratio": null});
    let context = Arc::new(adk_tool::SimpleToolContext::new("test-session".to_string()));
    let result = tool.execute(context, args).await;
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("GOOGLE_API_KEY is not set"), "Error should mention GOOGLE_API_KEY being missing, got: {}", msg);
            assert!(msg.contains("Vertex AI project ID (GOOGLE_CLOUD_PROJECT) is empty"), "Error should mention empty project ID, got: {}", msg);
        }
        _ => panic!("Expected error due to missing API key and Vertex project ID"),
    }
}
