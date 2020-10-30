pub mod gluedb;
pub mod mongo;
use super::*;
pub async fn initialize() {
    let _ = mongo::initialize().await;
    gluedb::initialize().await;
}
