use std::sync::Arc;
use adk_session::{CreateRequest, GetRequest, SessionService};
use std::collections::HashMap;

pub async fn ensure_session(
    sessions: &Arc<dyn SessionService>,
    app_name: &str,
    user_id: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    if sessions
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            num_recent_events: None,
            after: None,
        })
        .await
        .is_ok()
    {
        return Ok(());
    }

    sessions
        .create(CreateRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: Some(session_id.to_string()),
            state: HashMap::new(),
        })
        .await?;

    Ok(())
}