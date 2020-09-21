use super::*;
use mongodb::{options::ClientOptions, Client, Database};
use once_cell::sync::OnceCell;
use std::env;

static MONGO: OnceCell<Database> = OnceCell::new();

pub async fn initialize() {
    let source = "MONGO_INIT";
    let error = util::logger::error(source);

    if MONGO.get().is_some() {
        return;
    }

    // no one else has initialized it yet, so
    if let Ok(token) = env::var("MONGO_AUTH") {
        if let Ok(client_options) = ClientOptions::parse(token.as_str()).await {
            if let Ok(client) = Client::with_options(client_options) {
                let _ = MONGO.set(client.database("terminal"));
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

pub async fn get() -> Option<&'static Database> {
    let source = "MONGO_GET";
    let info = util::logger::info(source);

    MONGO.get().map(|db| {
        info("DB already initialized");
        db
    })
}
