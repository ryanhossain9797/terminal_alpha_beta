use crate::functions::*;
use mongodb::{
    options::ClientOptions,
    // options::FindOptions,
    Client,
};
use once_cell::sync::OnceCell;
use std::env;
static MONGO: OnceCell<Client> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<tokio::sync::Mutex<bool>> = OnceCell::new();

pub async fn initialize() {
    initialize_mongo().await;
}

async fn initialize_mongo() {
    let source = "MONGO_INIT";
    // it hasn't been initialized yet, so let's grab the lock & try to
    // initialize it
    let initializing_mutex = MONGO_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));
    // if initialized is true, then someone else initialized it already.
    let mut initialized = initializing_mutex.lock().await;
    if !*initialized {
        util::log_warning(source, "Not yet initialized");
        // no one else has initialized it yet, so
        if let Ok(token) = env::var("MONGO_AUTH") {
            if let Ok(client_options) = ClientOptions::parse(&token).await {
                if let Ok(client) = Client::with_options(client_options) {
                    if let Ok(_) = MONGO.set(client) {
                        *initialized = true;
                        util::log_info(source, "Initialized successfully");
                    }
                }
            }
        }
    }
    if !*initialized {
        util::log_error(source, "Initialization failure")
    }
}

pub async fn get_mongo() -> Option<&'static Client> {
    let source = "MONGO_GET";
    // this is racy, but that's OK: it's just a fast case
    let client_option = MONGO.get();
    if let Some(_) = client_option {
        util::log_info(source, "Already initialized");
        return client_option;
    }
    initialize_mongo().await;
    MONGO.get()
}
