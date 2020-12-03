use super::database::gluedb::*;
use colored::*;
use gluesql::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

///Returns a closure that logs the message with blue text
pub fn info(source: &str) -> impl Fn(&str) + '_ {
    move |msg: &str| println!("{}: {}", source.green(), msg.blue())
}
///Returns a closure that logs the message with yellow text
// pub fn warning(source: &str) -> impl Fn(&str) + '_ {
//     move |msg: &str| println!("{}: {}", source.green(), msg.yellow())
// }

///Returns a closure that logs the message with red text
pub fn error(source: &str) -> impl Fn(&str) + '_ {
    move |msg: &str| println!("{}: {}", source.green(), msg.red())
}
///Returns a closure that logs the message with white on purple text
pub fn status() -> impl Fn(&str) {
    move |msg: &str| show_status(msg)
}
///Logs the message with white on purple text
pub fn show_status(msg: &str) {
    println!("{}", msg.on_white().black());
}
///Logs the provided text to the `action_log.txt` file.  
///Used for when a message is unknown.
pub async fn log_message(processed_text: &str) -> anyhow::Result<()> {
    let source = "LOG_MESSAGE";
    let error = error(source);

    Ok(OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        //Open/Create the action_log.txt file with read, append, create options
        .open("action_log.txt")
        .map_err(|err| {
            error(format!("{}", err).as_str());
            anyhow::anyhow!(err)
        })?
        //Attempt to write to file
        .write((format!("{}\n", processed_text).as_str()).as_bytes())
        .map(|_| ())
        .map_err(|err| {
            error(format!("{}", err).as_str());
            anyhow::anyhow!(err)
        })?)
}

#[allow(dead_code)]
pub async fn log_message_db(message: &str) -> anyhow::Result<()> {
    let source = "GLUESQL_LOG";
    let error = error(source);
    let info = info(source);
    let sql = format!(
        "
        CREATE TABLE IF NOT EXISTS unintelligible_log (log TEXT);
        INSERT INTO unintelligible_log VALUES (\"{}\");
        ",
        message
    );

    let queries = parse(sql.as_str())?;

    let glue = GLUE
        .get()
        .ok_or_else(|| anyhow::anyhow!("gluedb not initialized"))?;

    if (*glue.lock().await)
        .execute(
            queries
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("no second query"))?,
        )
        .is_ok()
    {
        if (*glue.lock().await)
            .execute(
                queries
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("no second query"))?,
            )
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
