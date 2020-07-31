use super::*;
use mongodb::{
    options::ClientOptions,
    // options::FindOptions,
    Client,
    Database,
};
use once_cell::sync::Lazy;
use std::env;

static MONGO: Lazy<tokio::sync::Mutex<Option<Database>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

pub async fn initialize() {
    initialize_mongo().await;
}

async fn initialize_mongo() {
    let source = "MONGO_INIT";
    let error = util_service::make_error(source);

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
    let info = util_service::make_info(source);

    match &*MONGO.lock().await {
        Some(db) => {
            info("DB already initialized");
            Some(db.clone())
        }
        _ => None,
    }
}
