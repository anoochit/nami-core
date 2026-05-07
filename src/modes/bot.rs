use std::sync::Arc;

use adk_session::{CreateRequest, DeleteRequest, GetRequest, SessionService};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::runner::AgentRunner;

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

    log::info!("Starting namiClaw...");

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

async fn ensure_session(
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
            num_recent_events: Some(0),
            after: None,
        })
        .await
        .is_err()
    {
        sessions
            .create(CreateRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: Some(session_id.to_string()),
                state: Default::default(),
            })
            .await?;
    }
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

    ensure_session(&sessions, "telegram", &chat_id, &chat_id).await?;

    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;

    match runner.run(&chat_id, &chat_id, text).await {
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
