use std::sync::Arc;

use adk_session::{DeleteRequest, SessionService};
use teloxide::{prelude::*, utils::command::BotCommands};
use crate::modes::command_registry::CommandRegistry;
use crate::modes::slash_dispatcher::{self, SlashAction, SlashRequest};
use crate::runner::AgentRunner;
use crate::utils::{get_nami_dir, session};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Start the bot
    Start,
    /// Show help information
    Help,
    /// Clear the current session
    Clear,
}

pub async fn run_bot(
    runner: Arc<AgentRunner>,
    sessions: Arc<dyn SessionService>,
) -> anyhow::Result<()> {
    let bot = Bot::from_env();

    // Register commands for autocomplete
    bot.set_my_commands(Command::bot_commands()).await?;

    log::info!("Starting nami...");

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(Update::filter_message().endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![runner, sessions])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    _runner: Arc<AgentRunner>,
    sessions: Arc<dyn SessionService>,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id.to_string();
    log::info!("Received command: {:?} from {}", cmd, chat_id);

    match cmd {
        Command::Start | Command::Help => {
            bot.send_message(msg.chat.id, "👋 Hello!").await?;
        }
        Command::Clear => {
            sessions
                .delete(DeleteRequest {
                    app_name: "telegram".to_string(),
                    user_id: chat_id.clone(),
                    session_id: chat_id.clone(),
                })
                .await?;

            bot.send_message(msg.chat.id, "✅ Cleared").await?;
        }
    }

    Ok(())
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    runner: Arc<AgentRunner>,
    sessions: Arc<dyn SessionService>,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };
    let chat_id = msg.chat.id.to_string();
    log::info!("Received message from {}: {}", chat_id, text);

    session::ensure_session(&sessions, "telegram", &chat_id, &chat_id).await?;

    let resolved_text = if text.starts_with('/') {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).copied().unwrap_or("");

        let config_path = get_nami_dir().join("config.toml");
        let registry = CommandRegistry::load_from_config(&config_path.to_string_lossy()).unwrap_or_default();

        // Handle /new locally (create fresh session)
        if command == "/new" {
            sessions
                .delete(DeleteRequest {
                    app_name: "telegram".to_string(),
                    user_id: chat_id.clone(),
                    session_id: chat_id.clone(),
                })
                .await?;
session::ensure_session(&sessions, "telegram", &chat_id, &chat_id).await?;
            bot.send_message(msg.chat.id, "✅ New session started!").await?;
            return Ok(());
        }

        match slash_dispatcher::dispatch(SlashRequest {
            command,
            args,
            registry: &registry,
        }) {
            SlashAction::RunPrompt(prompt) => prompt,
            SlashAction::Reply(reply) => {
                bot.send_message(msg.chat.id, reply).await?;
                return Ok(());
            }
            SlashAction::PassThrough => text.to_string(),
        }
    } else {
        text.to_string()
    };

    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;

    match runner.run(&chat_id, &chat_id, &resolved_text).await {
        Ok(response) => {
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            log::error!("Error running agent: {:?}", e);
            bot.send_message(msg.chat.id, "❌ Sorry, an error occurred.")
                .await?;
        }
    }

    Ok(())
}
