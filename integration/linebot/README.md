# LINE Bot Integration Guide

Nami now supports integration with the LINE Messaging API. This allows you to interact with your AI agent directly through a LINE chat bot.

## Configuration

To use the LINE bot, you need to configure your credentials. You can do this during the initialization process or by manually editing your `.env` file.

### 1. Using `nami init`
Run the following command and follow the prompts:
```bash
cargo run -- init
```
You will be asked to enter your **LINE Channel Secret** and **LINE Channel Access Token**.

### 2. Manual Configuration
Add the following to your `.env` file:
```env
LINE_CHANNEL_SECRET=your_channel_secret_here
LINE_CHANNEL_ACCESS_TOKEN=your_channel_access_token_here
```

## Running the Bot

To start the LINE bot webhook server, run:
```bash
cargo run -- line
```
By default, the server runs on port `8080`. You can specify a different port using the `--port` flag:
```bash
cargo run -- line --port 9090
```

## Webhook Setup

Since LINE requires a public HTTPS endpoint for webhooks, you will need to expose your local server if you are developing locally.

1.  **Use ngrok**: Run ngrok to expose your port (e.g., 8080):
    ```bash
    ngrok http 8080
    ```
2.  **Configure LINE Developers Console**:
    *   Go to your Provider > Channel > **Messaging API** settings.
    *   Set the **Webhook URL** to: `https://<your-ngrok-id>.ngrok-free.app/callback`
    *   Enable **Use webhook**.
    *   Click **Verify** to ensure Nami is receiving the requests correctly.

## Implementation Details

The implementation is located in `src/modes/line.rs` and uses:
- **Axum**: For the HTTP server handling the `/callback` endpoint.
- **HMAC-SHA256**: For verifying the `x-line-signature` header to ensure requests are authentic.
- **AgentRunner**: To process incoming text and generate AI responses.
- **Reqwest**: To send reply messages back to the LINE API.
