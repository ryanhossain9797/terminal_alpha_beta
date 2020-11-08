use super::*;
use util::logger;

use anyhow::anyhow;

use once_cell::sync::Lazy;
use snips_nlu_lib::SnipsNluEngine;

const INTENTS_ALTERNATIVES: usize = 1;
const SLOTS_ALTERNATIVES: usize = 1;

///NLUENGINE: Snips NLU is used to pick actions when they don't match directly
pub static NLUENGINE: Lazy<Option<SnipsNluEngine>> = Lazy::new(|| {
    util::logger::show_status("\nLoading the nlu engine...");
    SnipsNluEngine::from_path("data/rootengine/").ok()
});

pub enum Intent {
    Acknowledgement,
    Chat,
    Search,
    Identify,
    Animation,
    Info { json: serde_json::Value },
    Notes,
    Corona,
    Reminder { json: serde_json::Value },
    Greet,
    About,
    Technology,
    Functions,
    Creator,
    Unknown,
}

fn parse(processed_text: &str) -> anyhow::Result<Option<(f32, String, serde_json::Value)>> {
    let result = (&*NLUENGINE)
        .as_ref()
        .ok_or_else(|| anyhow!("NLU engine inactive"))?
        .parse_with_alternatives(
            processed_text,
            None,
            None,
            INTENTS_ALTERNATIVES,
            SLOTS_ALTERNATIVES,
        )
        .map_err(|err| anyhow::anyhow!(format!("{}", err)))?;
    result
        .intent
        .intent_name
        .as_ref()
        .map(|intent_name| {
            Ok((
                result.intent.confidence_score,
                intent_name.clone(),
                serde_json::to_value(&result)?,
            ))
        })
        .transpose()
}

///Uses natural understanding to determine intent if no state is found
pub async fn detect(processed_text: &str) -> anyhow::Result<Option<Intent>> {
    let source = "NATURAL_ACTION_PICKER";
    let info = logger::info(source);

    parse(processed_text).map(|maybe_intent_data| {
        maybe_intent_data.and_then(|(confidence, intent_name, json)| {
            if confidence > 0.5 {
                use Intent::{
                    About, Acknowledgement, Animation, Chat, Corona, Creator, Functions, Greet,
                    Identify, Info, Notes, Reminder, Search, Technology, Unknown,
                };

                info(format!("intent is {}", intent_name).as_str());

                match intent_name.as_str() {
                    "acknowledgement" => Acknowledgement.into(),
                    "chat" => Chat.into(),
                    "search" => Search.into(),
                    "identify" => Identify.into(),
                    "animation" => Animation.into(),
                    "info" => Info { json }.into(),
                    "notes" => Notes.into(),
                    "corona" => Corona.into(),
                    "reminder" => Reminder { json }.into(),
                    "unknown" => Unknown.into(),
                    "greet" => Greet.into(),
                    "about" => About.into(),
                    "technology" => Technology.into(),
                    "functions" => Functions.into(),
                    "creator" => Creator.into(),
                    _ => None,
                }
            } else {
                None
            }
        })
    })
}
