pub mod mongo;
use super::*;
pub async fn initialize() {
    mongo::initialize_mongo().await;
}