use std::sync::Arc;
use adk_rust::prelude::*;
use adk_rust::Tool;
use adk_tool::{AdkError, tool};
use serde_json::Value;
use crate::tools::image_generator::image_generator_tools;

#[tokio::test]
async fn test_image_generator_no_api_key() {
    // Ensure GOOGLE_API_KEY is not set
    std::env::remove_var("GOOGLE_API_KEY");
    let tools = image_generator_tools(None);
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];
    // Call the tool with a dummy prompt, expecting an error about missing API key
    let args = serde_json::json!({"prompt": "a cat", "aspect_ratio": null});
    let result = tool.execute(Arc::new(MockContext::new()), args).await;
    match result {
        Err(AdkError::Tool(msg)) => assert!(msg.contains("GOOGLE_API_KEY environment variable not set")),
        _ => panic!("Expected error due to missing API key"),
    }
}

// Simple mock context that satisfies the ToolContext trait
struct MockContext;
impl MockContext {
    fn new() -> Self { MockContext }
}
impl ToolContext for MockContext {}
