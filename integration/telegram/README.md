# Telegram Bot Integration Guide

Nami now supports integration with the Telegram Bot API. This allows you to interact with your AI agent directly through a Telegram chat bot.

## Configuration

To use the Telegram bot, you need to configure your credentials. You can do this during the initialization process or by manually editing your `.env` file.

### 1. Using `nami init`
Run the following command and follow the prompts:
```bash
cargo run -- init
```
You will be asked to enter your **Telegram Bot Token**.

### 2. Manual Configuration
Add the following to your `.env` file:
```env
TELOXIDE_TOKEN=your_bot_token_here
```

## Running the Bot

To start the Telegram bot, run:
```bash
cargo run -- telegram
```

## Webhook vs. Polling

Telegram supports both polling and webhooks. By default, Nami uses long polling for simplicity in development.

- **Polling**: No public URL required. Nami continuously checks Telegram servers for new messages.
- **Webhooks** (Optional): If you prefer webhooks for lower latency, you would need to configure a public HTTPS endpoint and call the `setWebhook` method on the Telegram Bot API.

## Implementation Details

The implementation is located in `src/modes/telegram.rs` and uses:
- **Telegram Bot API**: For sending and receiving messages.
- **AgentRunner**: To process incoming text and generate AI responses.
- **Reqwest/Custom Client**: To interact with Telegram's HTTPS API.
