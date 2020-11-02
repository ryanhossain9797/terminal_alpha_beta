pub mod discord;
mod shared_utils;
pub mod telegram;
use super::*;
use flume::Sender;
use std::sync::Arc;

///Just an entry point to start the telegram api.
pub async fn run_telegram(
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> anyhow::Result<!> {
    telegram::main(sender).await
}

///Just an entry point to start the discord api.
pub async fn run_discord(
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> anyhow::Result<!> {
    discord::main(sender).await
}
