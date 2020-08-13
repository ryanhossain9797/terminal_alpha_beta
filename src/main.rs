#![feature(async_closure)]
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

    //Wait for tasks to finish,
    //Which is hopefully never, because that would mean it crashed.
    futures::future::join_all(vec![
        //Spawn a task for telegram
        task::spawn(async {
            run_telegram().await;
        }),
        //Spawn a task for discord
        task::spawn(async {
            run_discord().await;
        }),
    ])
    .await;
}

// #[allow(dead_code)]
// async fn download_file(url: String) -> Result<String, Box<dyn std::error::Error>> {
//     let mut response = reqwest::get(&url).await?;
//     let mut file = tokio::fs::File::open("temp/file.gif").await?;
//     while let Some(chunk) = response.chunk().await? {
//         file.write_all(&chunk).await?;
//     }
//     Ok("temp/file.gif".to_string())
// }
