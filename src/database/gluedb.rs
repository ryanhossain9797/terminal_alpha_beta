use async_std::sync::Mutex;
use gluesql::*;
use once_cell::sync::OnceCell;

pub static GLUE: OnceCell<Mutex<Glue>> = OnceCell::new();

pub async fn initialize() -> anyhow::Result<()> {
    // let source = "GLUESQL_INIT";

    if GLUE.get().is_some() {
        return Ok(());
    }

    #[allow(clippy::map_err_ignore)]
    GLUE.set(Mutex::new(Glue::new(
        #[allow(clippy::map_err_ignore)]
        SledStorage::new("data/gluedb")
            .map_err(|_| anyhow::anyhow!("sled storage initilization failed"))?,
    )))
    .map_err(|_| anyhow::anyhow!("already initialized"))
}
