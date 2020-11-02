#![feature(async_closure)]
#![feature(never_type)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]
mod api;
mod clients;
mod database;
mod handlers;
mod repositories;
mod services;
mod util;
use async_std::task;
use clients::*;
use dotenv::dotenv;
use flume::{Receiver, Sender};
use services::*;
use std::sync::Arc;

#[async_std::main]
async fn main() {
    let source = "MAIN";
    let error = util::logger::error(source);
    let status = util::logger::status();
    {
        //---Load up all the ENV variables from .env file
        dotenv().expect("Couldn't load environment variables");
        status("Starting up Terminal Alpha Beta\n");
        status("-----Starting TELEGRAM and DISCORD-----\n"); //---Prints the Date of compilation, added at compile time
        if let Some(date) = option_env!("COMPILED_AT") {
            status(&format!("Compile date {}\n", date));
        }
        status("Initializing everything");
        handlers::initialize().await;
        let _ = database::initialize().await;
        status("\nInitialized Everything\n");
    }
    let (sender, receiver) = handlers::init_sender().await;

    //Wait for tasks to finish,
    //Which is hopefully never, because that would mean it crashed.
    let clients_result = futures::future::try_join_all(vec![
        //Spawn task to handle inbound Updates from clients
        task::spawn(async { services(receiver).await }),
        //Spawn a task for clients
        task::spawn(async { clients(sender).await }),
    ])
    .await;

    if let Err(err) = clients_result {
        error(&format!("One or more services have failed {}", err));
    }
}

async fn services(
    receiver: Receiver<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> anyhow::Result<!> {
    futures::future::try_join_all(vec![
        //Spawn a task to receive updates
        task::spawn(async { handlers::receiver(receiver).await }),
        //Spawn a task to spawn reminder notifications
        task::spawn(async { handlers::reminder_service().await }),
        //Spawn a task to clean up expired states
        // task::spawn(async { handlers::state_expiry_service().await }),
    ])
    .await?;

    Err(anyhow::anyhow!("Services failed"))
}

async fn clients(
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> anyhow::Result<!> {
    let telegram_sender = sender.clone();
    let discord_sender = sender;

    futures::future::try_join_all(vec![
        task::spawn(async { run_telegram(telegram_sender).await }),
        task::spawn(async { run_discord(discord_sender).await }),
    ])
    .await?;

    Err(anyhow::anyhow!("Clients failed"))
}
