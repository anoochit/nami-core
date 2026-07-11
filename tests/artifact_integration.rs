use std::sync::Arc;
use adk_artifact::{FileArtifactService, ScopedArtifacts, ArtifactService};
use adk_core::{Artifacts, Part};

#[tokio::test]
async fn test_file_artifact_service_lifecycle() -> anyhow::Result<()> {
    // Create a temporary directory path using a unique UUID
    let temp_dir = std::env::temp_dir().join(format!("nami-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    
    let service = Arc::new(FileArtifactService::new(&temp_dir)?);

    // Verify health check
    assert!(service.health_check().await.is_ok());

    // Wrap with ScopedArtifacts
    let scoped = ScopedArtifacts::new(
        service.clone(),
        "nami-test-app".to_string(),
        "test-user".to_string(),
        "test-session".to_string(),
    );

    // 1. Save artifact (text)
    let text_content = "Hello, Nami Artifacts!".to_string();
    let v1 = scoped.save(
        "welcome.txt",
        &Part::Text { text: text_content.clone() }
    ).await?;
    assert_eq!(v1, 1);

    // 2. Save new version (auto-increments)
    let text_content_v2 = "Hello, Nami Artifacts! Version 2".to_string();
    let v2 = scoped.save(
        "welcome.txt",
        &Part::Text { text: text_content_v2.clone() }
    ).await?;
    assert_eq!(v2, 2);

    // 3. List artifacts
    let files = scoped.list().await?;
    assert!(files.contains(&"welcome.txt".to_string()));

    // 4. Load latest version
    let loaded = scoped.load("welcome.txt").await?;
    match loaded {
        Part::Text { text } => {
            assert_eq!(text, text_content_v2);
        }
        _ => panic!("Expected text part"),
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}
