use super::*;
use mongodb::{options::ClientOptions, Client, Database};
use once_cell::sync::OnceCell;
use std::env;

static MONGO_AUTH: &str = "MONGO_AUTH";

static MONGO: OnceCell<Database> = OnceCell::new();

pub async fn initialize() -> anyhow::Result<()> {
    //let source = "MONGO_INIT";

    if MONGO.get().is_some() {
        return Ok(());
    }

    // no one else has initialized it yet, so
    MONGO
        .set(
            Client::with_options(ClientOptions::parse(env::var(MONGO_AUTH)?.as_str()).await?)?
                .database("terminal"),
        )
        .map_err(|db| anyhow::anyhow!(format!("Already initialized {}", db.name())))
}

pub async fn get() -> Option<&'static Database> {
    let source = "MONGO_GET";
    let info = util::logger::info(source);

    MONGO.get().map(|db| {
        info("DB already initialized");
        db
    })
}
