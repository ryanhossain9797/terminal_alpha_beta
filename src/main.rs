#![feature(async_closure)]
#![feature(never_type)]
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
use services::*;

#[async_std::main]
async fn main() {
    let source = "MAIN";
    let error = util::logger::make_error(source);
    let status = util::logger::make_status();

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
        handlers::initialize();
        database::initialize().await;

        status("\nInitialized Everything\n");
    }

    //Wait for tasks to finish,
    //Which is hopefully never, because that would mean it crashed.
    let clients_result = futures::future::try_join_all(vec![
        //Spawn a task for telegram
        task::spawn(async {
            run_telegram().await;
            return Result::Err::<!, &str>("Telegram failed");
        }),
        //Spawn a task for discord
        task::spawn(async {
            run_discord().await;
            return Result::Err::<!, &str>("Discord failed");
        }),
    ])
    .await;

    if let Err(err) = clients_result {
        error(&format!("One or more clients have failed {}", err));
    }
}
