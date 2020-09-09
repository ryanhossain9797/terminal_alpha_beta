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
        status("-----Starting TELEGRAM and DISCORD-----\n");

        //---Prints the Date of compilation, added at compile time
        if let Some(date) = option_env!("COMPILED_AT") {
            status(&format!("Compile date {}\n", date));
        }
        status("Initializing everything");

        clients::initialize();
        handlers::initialize().await;
        database::initialize().await;

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
) -> Result<!, &'static str> {
    let _ = futures::future::try_join_all(vec![
        //Spawn a task to receive updates
        task::spawn(async {
            handlers::receiver(receiver).await;
            Err::<!, &str>("Receiver Failed")
        }),
        //Spawn a task to receive updates
        task::spawn(async {
            handlers::reminder_service().await;
            Err::<!, &str>("Reminder Failed")
        }),
    ])
    .await;
    Err::<!, &'static str>("Services failed")
}

async fn clients(
    sender: Sender<(Arc<Box<dyn handlers::BotMessage>>, String)>,
) -> Result<!, &'static str> {
    let telegram_sender = sender.clone();
    let discord_sender = sender;
    let _ = futures::future::try_join_all(vec![
        //Spawn a task for telegram
        task::spawn(async {
            run_telegram(telegram_sender).await;
            Err::<!, &str>("Telegram failed")
        }),
        //Spawn a task for discord
        task::spawn(async {
            run_discord(discord_sender).await;
            Err::<!, &str>("Discord failed")
        }),
    ])
    .await;
    Err::<!, &'static str>("Clients failed")
}
