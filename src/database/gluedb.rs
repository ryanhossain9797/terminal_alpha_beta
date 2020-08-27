use super::*;
use async_std::sync::Mutex;
use gluesql::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;

pub static GLUE: Lazy<Mutex<Option<RefCell<Glue>>>> = Lazy::new(|| Mutex::new(None));

pub async fn initialize_glue() {
    let source = "GLUESQL_INIT";
    let info = util::logger::info_logger(source);
    let error = util::logger::error_logger(source);
    match SledStorage::new("data/gluedb") {
        Ok(storage) => {
            let glue = Glue::new(storage);
            *GLUE.lock().await = Some(RefCell::new(glue));
        }
        Err(err) => error(&format!("{}", err)),
    }
}

pub async fn log_message(message: &str) -> anyhow::Result<()> {
    let source = "GLUESQL_LOG";
    let error = util::logger::error_logger(source);
    let info = util::logger::info_logger(source);
    let sql = &format!(
        "
        CREATE TABLE IF NOT EXISTS unintelligible_log (log TEXT);
        INSERT INTO unintelligible_log VALUES (\"{}\");
        ",
        message
    );

    let queries = parse(sql)?;
    let glue_mutex = GLUE.lock().await;
    let glue = glue_mutex.as_ref().ok_or_else(|| anyhow::anyhow!(""))?;

    if (*glue.borrow_mut())
        .execute(queries.get(0).expect("there is no first query"))
        .is_ok()
    {
        if (*glue.borrow_mut())
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
