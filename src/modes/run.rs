use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_rust::session::{CreateRequest, SessionService};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) async fn run_direct(agent: Arc<dyn Agent>, prompt: &str) -> anyhow::Result<()> {
    let app_name = "cli";
    let user_id = "default_user";

    let session_service = Arc::new(InMemorySessionService::new());

    // Create session using the correct Rust ADK API
    let session = session_service
        .create(CreateRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: None, // let the service generate an ID
            state: HashMap::new(),
        })
        .await?;

    let session_id = session.id(); // use the generated session ID

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(session_service)
        .build()?;

    let user_content = Content::new("user").with_text(prompt);

    let mut stream = runner.run_str(user_id, &session_id, user_content).await?;
    let mut full_response = String::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Some(content) = &event.llm_response.content {
                    for part in &content.parts {
                        if let Some(text) = &part.text() {
                            print!("{}", text);
                            full_response.push_str(text);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error running agent: {:?}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
