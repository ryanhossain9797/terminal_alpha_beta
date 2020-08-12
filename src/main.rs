#![feature(async_closure)]
mod api;
mod clients;
mod database;
mod handlers;
mod repositories;
mod services;
mod util;
use clients::*;
use dotenv::dotenv;
use services::*;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
fn main() {
    let mut rt = Runtime::new().expect("Couldn't set up tokio runtime");
    rt.block_on(async {
        {
            //---Load up all the ENV variables from .env file
            dotenv().expect("Couldn't load environment variables");
            let status = util::logger::make_status();
            status("Starting up Terminal Alpha Beta");
            status("-----Starting TELEGRAM and DISCORD-----\n");
            //---Prints the Date of compilation, added at compile time
            if let Some(date) = option_env!("COMPILED_AT") {
                status(&format!("Compile date {}", date));
            }
            status("Initializing everything");
            clients::initialize();
            handlers::initialize();
            database::initialize().await;
            status("\nInitialized Everything\n");
        }

        let (sender, receiver) = mpsc::channel::<(Box<dyn handlers::BotMessage>, String)>(100);
        let message_handler = tokio::spawn(async move {
            handlers::distributor_new(receiver).await;
        });

        let telegram_sender = sender.clone();
        let discord_sender = sender.clone();
        drop(sender);
        //Wait for tasks to finish,
        //Which is hopefully never, because that would mean it crashed.
        futures::future::join_all(vec![
            message_handler,
            //Spawn a task for telegram
            tokio::spawn(async {
                run_telegram(telegram_sender).await;
            }),
            //Spawn a task for discord
            tokio::spawn(async {
                run_discord(discord_sender).await;
            }),
        ])
        .await;
    });
}

#[allow(dead_code)]
async fn download_file(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut response = reqwest::get(&url).await?;
    let mut file = tokio::fs::File::open("temp/file.gif").await?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok("temp/file.gif".to_string())
}
