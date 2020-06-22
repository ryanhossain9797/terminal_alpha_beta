use super::*;
pub mod discord;
pub mod telegram;
use discord::*;
use telegram::*;

extern crate lazy_static;

use telegram::API;

///Just an entry point to start the telegram api.
pub async fn run_telegram() {
    telegram_main().await;
}
///Just an entry point to start the discord api.
pub async fn run_discord() {
    discord_main().await;
}

///Any initialization required for setting up the Clients should go here
pub fn initialize() {
    //---Start the telegram API
    lazy_static::initialize(&API);
}
