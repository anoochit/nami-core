use std::sync::Arc;
use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Router,
};
use line_bot_sdk_rust::{
    client::LINE,
    line_messaging_api::{
        apis::MessagingApiApi,
        models::{Message, TextMessage, ReplyMessageRequest},
    },
    line_webhook::models::{CallbackRequest, Event, MessageContent, Source},
    parser::signature::validate_signature,
};
use crate::modes::command_registry::CommandRegistry;
use crate::modes::slash_dispatcher::{self, SlashAction, SlashRequest};
use crate::runner::AgentRunner;
use crate::utils::get_nami_dir;

struct AppState {
    runner: Arc<AgentRunner>,
    line_client: LINE,
    channel_secret: String,
}

pub async fn run_line(
    runner: Arc<AgentRunner>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let channel_secret = std::env::var("LINE_CHANNEL_SECRET")
        .context("LINE_CHANNEL_SECRET must be set")?;
    let channel_access_token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN")
        .context("LINE_CHANNEL_ACCESS_TOKEN must be set")?;

    let line_client = LINE::new(channel_access_token);
    let state = Arc::new(AppState {
        runner,
        line_client,
        channel_secret,
    });

    let app = Router::new()
        .route("/callback", post(handle_callback))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("LINE Bot Webhook server starting on http://{}", addr);
    println!("Press Ctrl+C to stop\n");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_callback(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let signature = headers
        .get("x-line-signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Use SDK to verify signature
    let body_str = std::str::from_utf8(&body).unwrap_or("");
    if !validate_signature(
        &state.channel_secret,
        signature,
        body_str,
    ) {
        log::warn!("Invalid LINE signature");
        return StatusCode::UNAUTHORIZED;
    }

    let request: CallbackRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => {
            log::error!("Failed to parse LINE webhook: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    for event in request.events {
        if let Event::MessageEvent(msg_event) = event {
            if let MessageContent::TextMessageContent(text_msg) = *msg_event.message {
                let user_id = match msg_event.source.as_deref() {
                    Some(Source::UserSource(s)) => s.user_id.clone(),
                    Some(Source::GroupSource(s)) => s.user_id.clone(),
                    Some(Source::RoomSource(s)) => s.user_id.clone(),
                    _ => None,
                }.unwrap_or_else(|| "unknown".to_string());

                let reply_token = msg_event.reply_token;

                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = process_message(state_clone, user_id, text_msg.text, reply_token).await {
                        log::error!("Error processing LINE message: {:?}", e);
                    }
                });
            }
        }
    }

    StatusCode::OK
}

async fn process_message(
    state: Arc<AppState>,
    user_id: String,
    text: String,
    reply_token: Option<String>,
) -> anyhow::Result<()> {
    log::info!("Received LINE message from {}: {}", user_id, text);

    let resolved_text = if text.starts_with('/') {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).copied().unwrap_or("");

        let config_path = get_nami_dir().join("config.toml");
        let registry = CommandRegistry::load_from_config(&config_path.to_string_lossy()).unwrap_or_default();

        match slash_dispatcher::dispatch(SlashRequest {
            command,
            args,
            registry: &registry,
        }) {
            SlashAction::RunPrompt(prompt) => prompt,
            SlashAction::Reply(reply) => {
                if let Some(token) = reply_token {
                    let reply_request = ReplyMessageRequest {
                        reply_token: token,
                        messages: vec![Message::TextMessage(TextMessage {
                            text: reply,
                            emojis: None,
                            quote_token: None,
                            quick_reply: None,
                            sender: None,
                        })],
                        notification_disabled: None,
                    };
                    state.line_client.messaging_api_client.reply_message(reply_request).await
                        .map_err(|e| anyhow::anyhow!("Failed to reply to LINE: {:?}", e))?;
                }
                return Ok(());
            }
            SlashAction::PassThrough => text,
        }
    } else {
        text
    };

    // Run agent
    let response = state.runner.run(&user_id, &user_id, &resolved_text).await?;

    // Reply if token is available
    if let Some(token) = reply_token {
        let reply_request = ReplyMessageRequest {
            reply_token: token,
            messages: vec![Message::TextMessage(TextMessage {
                text: response,
                emojis: None,
                quote_token: None,
                quick_reply: None,
                sender: None,
            })],
            notification_disabled: None,
        };

        state.line_client.messaging_api_client.reply_message(reply_request).await
            .map_err(|e| anyhow::anyhow!("Failed to reply to LINE: {:?}", e))?;
    }

    Ok(())
}
