use super::*;
use async_std::sync::Mutex;
use gluesql::*;
use once_cell::sync::OnceCell;

pub static GLUE: OnceCell<Mutex<Glue>> = OnceCell::new();

pub async fn initialize() {
    let source = "GLUESQL_INIT";
    let error = util::logger::error(source);

    if GLUE.get().is_some() {
        return;
    }
    match SledStorage::new("data/gluedb") {
        Ok(storage) => {
            let glue = Glue::new(storage);
            let _ = GLUE.set(Mutex::new(glue));
        }
        Err(err) => error(format!("{}", err).as_str()),
    }
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
