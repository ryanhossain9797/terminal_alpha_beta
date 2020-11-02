// use async_std::sync::RwLock;
// use futures::stream::{FuturesUnordered, Stream};
// use once_cell::sync::Lazy;
// use std::future::Future;
// use std::pin::Pin;
// use std::time::Duration;

// pub static CLEAN_QUEUE: Lazy<
//     RwLock<FuturesUnordered<Pin<Box<dyn Future<Output = ()> + Sync + Send>>>>,
// > = Lazy::new(|| RwLock::new(FuturesUnordered::new()));

// pub async fn service() -> anyhow::Result<!> {
//     loop {
//         let mut queue = CLEAN_QUEUE.write().await;

//     }
//     Err(anyhow::anyhow!("Expiry service crashed"))
// }
