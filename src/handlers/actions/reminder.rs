use super::*;
use flume::{Receiver, Sender};
use once_cell::sync::Lazy;
use serde_json::Value;
use serde_query::{DeserializeQuery, Query};

type TimedMessage = (Box<dyn BotMessage>, Duration, String);

static REMINDER: Lazy<(Sender<TimedMessage>, Receiver<TimedMessage>)> =
    Lazy::new(|| flume::bounded::<TimedMessage>(10));

#[derive(DeserializeQuery, Copy, Clone)]
struct ReminderDuration {
    #[query(".value.days")]
    days: u64,

    #[query(".value.hours")]
    hours: u64,

    #[query(".value.minutes")]
    minutes: u64,

    #[query(".value.seconds")]
    seconds: u64,
}

pub async fn start(bot_message: Box<dyn BotMessage>, json: serde_json::Value) {
    let source = "START_REMINDER";
    let info = util::logger::info(source);
    let maybe_title_pass = duration_reminder_retriever(&json);

    if let Some((reminder, duration)) = maybe_title_pass {
        info(reminder.as_str());

        let delay = Duration::from_secs(
            ((duration.days * 24 + duration.hours) * 60 + duration.minutes) * 60 + duration.seconds,
        );

        let confirmation_template = responses::load_text("reminder-confirmation")
            .unwrap_or_else(|| "(fallback) Reminder set: {reminder}".to_string());

        bot_message
            .send_message(
                confirmation_template
                    .replace("{reminder}", reminder.as_str())
                    .into(),
            )
            .await;

        let _ = (*REMINDER)
            .0
            .send_async((bot_message, delay, reminder))
            .await;
    } else {
        bot_message
            .send_message(responses::load("reminder-fail").into())
            .await
    }
}

///Retrieves the title and pass for the info intent.  
///Parses the intent JSON.
fn duration_reminder_retriever(json: &serde_json::Value) -> Option<(String, ReminderDuration)> {
    let mut maybe_reminder: Option<&str> = None;
    let mut maybe_duration: Option<ReminderDuration> = None;
    let val = &json["slots"];
    if let Value::Array(values) = val {
        for slot in values {
            if let Value::String(entity) = &slot["slotName"] {
                if let Value::String(value) = &slot["rawValue"] {
                    //If slotName is title
                    if entity == &String::from("reminder") {
                        maybe_reminder = Some(&value);
                    //If slotName is pass
                    } else if entity == &String::from("duration") {
                        if let Ok(duration_string) = serde_json::from_str::<Query<ReminderDuration>>(
                            format!("{}", slot).as_str(),
                        ) {
                            maybe_duration = Some(duration_string.into());
                        }
                    }
                }
            }
        }
    }
    if let (Some(reminder), Some(duration)) = (maybe_reminder, maybe_duration) {
        return Some((reminder.to_string(), duration));
    }

    None
}

pub async fn service() -> anyhow::Result<!> {
    let body_template = responses::load_text("reminder-body")
        .unwrap_or_else(|| "(fallback) Reminder: {reminder}".to_string());
    while let Ok((bot_message, delay, reminder)) = (*REMINDER).1.recv_async().await {
        let template = body_template.clone();
        task::spawn(async move {
            task::sleep(delay).await;
            bot_message
                .send_message(template.replace("{reminder}", reminder.as_str()).into())
                .await;
        });
    }
    Err(anyhow::anyhow!("Reminder service crashed"))
}
