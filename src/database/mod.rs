pub mod gluedb;
pub mod mongo;
use super::*;
pub async fn initialize() {
    mongo::initialize_mongo().await;
    gluedb::initialize_glue().await;
}
