use async_std::sync::RwLock;
use futures::stream::FuturesUnordered;
use once_cell::sync::Lazy;
use std::time::Duration;

// pub static CLEAN_QUEUE: Lazy<RwLock<FuturesUnordered<()>>> =
//     Lazy::new(|| RwLock::new(FuturesUnordered::new()));

pub async fn service() -> anyhow::Result<!> {
    let runner = Some(1);
    while let Some(1) = runner {
        async_std::task::sleep(Duration::from_secs(10)).await;
        println!("expiry service");
    }
    Err(anyhow::anyhow!("Expiry service crashed"))
}
