pub mod discord;
mod shared_utils;
pub mod telegram;
use super::*;
use discord::*;
use flume::Sender;
use once_cell::sync::Lazy;
use std::sync::Arc;
use telegram::*;

use telegram::API;

///Just an entry point to start the telegram api.
pub async fn run_telegram(sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>) {
    telegram_main(sender).await;
}
///Just an entry point to start the discord api.
pub async fn run_discord(sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>) {
    discord_main(sender).await;
}

///Any initialization required for setting up the Clients should go here
pub fn initialize() {
    //---Start the telegram API
    Lazy::force(&API);
}
