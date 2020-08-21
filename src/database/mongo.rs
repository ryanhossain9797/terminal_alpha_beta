use super::*;
use async_std::sync::Mutex;
use mongodb::{options::ClientOptions, Client, Database};
use once_cell::sync::Lazy;
use std::env;

static MONGO: Lazy<Mutex<Option<Database>>> = Lazy::new(|| Mutex::new(None));

pub async fn initialize_mongo() {
    let source = "MONGO_INIT";
    let error = util::logger::make_error(source);

    // no one else has initialized it yet, so
    if let Ok(token) = env::var("MONGO_AUTH") {
        if let Ok(client_options) = ClientOptions::parse(&token).await {
            if let Ok(client) = Client::with_options(client_options) {
                *MONGO.lock().await = Some(client.database("terminal"));
            } else {
                error("Couldn't initialize client");
            }
        } else {
            error("Couldn't parse db token");
        }
    } else {
        error("Couldn't find DB auth key");
    }
}

pub async fn get_mongo() -> Option<Database> {
    let source = "MONGO_GET";
    let info = util::logger::make_info(source);

    match &*MONGO.lock().await {
        Some(db) => {
            info("DB already initialized");
            Some(db.clone())
        }
        _ => None,
    }
}
