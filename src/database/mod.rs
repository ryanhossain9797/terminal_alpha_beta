pub mod gluedb;
pub mod mongo;
use super::*;
pub async fn initialize() {
    mongo::initialize().await;
    gluedb::initialize().await;
}
