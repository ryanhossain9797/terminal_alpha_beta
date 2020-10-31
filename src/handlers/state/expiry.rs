use std::time::Duration;

pub async fn service() -> anyhow::Result<!> {
    loop {
        async_std::task::sleep(Duration::from_secs(10)).await;
        println!("state cleaner active");
    }
    Err(anyhow::anyhow!("Reminder service crashed"))
}
