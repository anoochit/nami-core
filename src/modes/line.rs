use std::sync::Arc;
use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use crate::runner::AgentRunner;

#[derive(Debug, Deserialize)]
pub struct LineWebhook {
    pub _destination: String,
    pub events: Vec<LineEvent>,
}

#[derive(Debug, Deserialize)]
pub struct LineEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message: Option<LineMessage>,
    pub source: LineSource,
    #[serde(rename = "replyToken")]
    pub reply_token: Option<String>,
    pub _mode: String,
    pub _timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct LineMessage {
    pub _id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LineSource {
    #[serde(rename = "type")]
    pub _source_type: String,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "groupId")]
    pub _group_id: Option<String>,
    #[serde(rename = "roomId")]
    pub _room_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct LineReply {
    #[serde(rename = "replyToken")]
    reply_token: String,
    messages: Vec<LineReplyMessage>,
}

#[derive(Debug, Serialize)]
struct LineReplyMessage {
    #[serde(rename = "type")]
    message_type: String,
    text: String,
}

struct AppState {
    runner: Arc<AgentRunner>,
    channel_secret: String,
    channel_access_token: String,
}

pub async fn run_line(
    runner: Arc<AgentRunner>,
    port: u16,
) -> anyhow::Result<()> {
    let channel_secret = std::env::var("LINE_CHANNEL_SECRET")
        .expect("LINE_CHANNEL_SECRET must be set");
    let channel_access_token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN")
        .expect("LINE_CHANNEL_ACCESS_TOKEN must be set");

    let state = Arc::new(AppState {
        runner,
        channel_secret,
        channel_access_token,
    });

    let app = Router::new()
        .route("/callback", post(handle_callback))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("LINE Bot Webhook server starting on http://localhost:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body_str: String,
) -> StatusCode {
    // 1. Verify signature
    let signature = match headers.get("x-line-signature").and_then(|h| h.to_str().ok()) {
        Some(s) => s,
        None => return StatusCode::BAD_REQUEST,
    };

    if !verify_signature(&state.channel_secret, &body_str, signature) {
        log::warn!("Invalid LINE signature");
        return StatusCode::UNAUTHORIZED;
    }

    // 2. Parse JSON
    let webhook: LineWebhook = match serde_json::from_str(&body_str) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to parse LINE webhook: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // 3. Handle events
    for event in webhook.events {
        if event.event_type == "message" {
            if let Some(msg) = event.message {
                if msg.message_type == "text" {
                    if let Some(text) = msg.text {
                        let user_id = event.source.user_id.clone().unwrap_or_else(|| "unknown".to_string());
                        let reply_token = event.reply_token.clone();

                        let state_clone = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = process_message(state_clone, user_id, text, reply_token).await {
                                log::error!("Error processing LINE message: {}", e);
                            }
                        });
                    }
                }
            }
        }
    }

    StatusCode::OK
}

fn verify_signature(secret: &str, body: &str, signature: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    
    let expected_signature = general_purpose::STANDARD.encode(code_bytes);
    expected_signature == signature
}

async fn process_message(
    state: Arc<AppState>,
    user_id: String,
    text: String,
    reply_token: Option<String>,
) -> anyhow::Result<()> {
    log::info!("Received LINE message from {}: {}", user_id, text);

    // Run agent
    let response = state.runner.run(&user_id, &user_id, &text).await?;

    // Reply if token is available
    if let Some(token) = reply_token {
        reply_to_line(&state.channel_access_token, token, response).await?;
    }

    Ok(())
}

async fn reply_to_line(
    access_token: &str,
    reply_token: String,
    text: String,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let reply = LineReply {
        reply_token,
        messages: vec![LineReplyMessage {
            message_type: "text".to_string(),
            text,
        }],
    };

    let res = client
        .post("https://api.line.me/v2/bot/message/reply")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&reply)
        .send()
        .await?;

    if !res.status().is_success() {
        let err_body = res.text().await?;
        log::error!("Failed to reply to LINE: {}", err_body);
        return Err(anyhow::anyhow!("LINE API error: {}", err_body));
    }

    Ok(())
}
