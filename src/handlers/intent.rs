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

fn parse(processed_text: &str) -> anyhow::Result<(f32, String, serde_json::Value)> {
    Ok({
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
        (
            result.intent.confidence_score,
            result
                .intent
                .intent_name
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no intent name".to_string()))?
                .clone(),
            serde_json::to_value(&result)?,
        )
    })
}

///Uses natural understanding to determine intent if no state is found
pub async fn detect(processed_text: &str) -> anyhow::Result<Option<Intent>> {
    let source = "NATURAL_ACTION_PICKER";
    let info = logger::info(source);

    //---Stuff required to run the NLU engine to get an intent
    if let Ok((confidence, intent_name, json)) = parse(processed_text) {
        //Only valid if confidence greater than 0.5
        if confidence > 0.5 {
            use Intent::{
                About, Animation, Chat, Corona, Creator, Functions, Greet, Identify, Info, Notes,
                Reminder, Search, Technology, Unknown,
            };

            info(format!("intent is {}", intent_name).as_str());

            Ok(match intent_name.as_str() {
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
            })
        } else {
            logger::log_message(processed_text).await?;
            Err(anyhow::anyhow!("No intent with sufficient confidence"))
        }
    } else {
        logger::log_message(processed_text).await?;
        Err(anyhow::anyhow!("Intent matching failed"))
    }
}
