extern crate snips_nlu_lib;
mod clients;
mod database;
mod functions;
mod handlers;
use clients::*;
use dotenv::dotenv;
use tokio::prelude::*;

#[tokio::main]
async fn main() {
    {
        //---Load up all the ENV variables from .env file
        dotenv().ok();
        println!("Starting up Terminal Alpha Beta, compiled at");
        println!("-----Starting TELEGRAM and DISCORD-----\n");
        //---Prints the Date of compilation, added at compile time
        if let Some(date) = option_env!("COMPILED_AT") {
            println!("Compile date {}", date);
        }
        println!("Initializing everything");
        clients::initialize();
        handlers::initialize();
        database::initialize().await;
        println!("\nInitialized Everything\n");
    }
    //Wait tasks to finish,
    //Which is hoepfully never, because that would mean it crashed.
    futures::future::join_all(vec![
        //Spawn a task for telegram
        tokio::spawn(async move {
            run_telegram().await;
        }),
        //Spawn a task for discord
        tokio::spawn(async move {
            run_discord().await;
        }),
    ])
    .await;
    //tokio LocalSet based approach for older non-send telegram
    // let local = tokio::task::LocalSet::new();
    // local
    //     .run_until(async move {
    //         let tasks = vec![
    //             //---A task for telegram
    //             tokio::task::spawn_local(async move {
    //                 run_telegram().await;
    //             }),
    //             //---A task for discord
    //             tokio::task::spawn_local(async move {
    //                 run_discord().await;
    //             }),
    //         ];
    //         //---And run them, wait for them to finish,
    //         //---Which is hopefully never, because that would mean it crashed.
    //         futures::future::join_all(tasks).await;
    //     })
    //     .await;
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
