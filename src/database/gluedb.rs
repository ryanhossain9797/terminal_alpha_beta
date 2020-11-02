use super::*;
use async_std::sync::Mutex;
use gluesql::*;
use once_cell::sync::OnceCell;

pub static GLUE: OnceCell<Mutex<Glue>> = OnceCell::new();

pub async fn initialize() -> anyhow::Result<()> {
    // let source = "GLUESQL_INIT";

    if GLUE.get().is_some() {
        return Ok(());
    }

    GLUE.set(Mutex::new(Glue::new(
        SledStorage::new("data/gluedb")
            .map_err(|_| anyhow::anyhow!("sled storage initilization failed"))?,
    )))
    .map_err(|_| anyhow::anyhow!("already initialized"))
}

#[allow(dead_code)]
pub async fn log_message(message: &str) -> anyhow::Result<()> {
    let source = "GLUESQL_LOG";
    let error = util::logger::error(source);
    let info = util::logger::info(source);
    let sql = &format!(
        "
        CREATE TABLE IF NOT EXISTS unintelligible_log (log TEXT);
        INSERT INTO unintelligible_log VALUES (\"{}\");
        ",
        message
    );

    let queries = parse(sql)?;
    let glue_mutex = GLUE.get();
    let glue = glue_mutex.ok_or_else(|| anyhow::anyhow!(""))?;
    if (*glue.lock().await)
        .execute(queries.get(0).expect("there is no first query"))
        .is_ok()
    {
        if (*glue.lock().await)
            .execute(queries.get(1).expect("there is no second query"))
            .is_ok()
        {
        } else {
            error("Log insertion error");
            return Err(anyhow::anyhow!("Log insertion error"));
        }
    } else {
        error("Table creation error");
        return Err(anyhow::anyhow!("Table creation error"));
    }
    info("Logged Successfully");
    Ok(())
}
