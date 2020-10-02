use super::*;

use anyhow::anyhow;

use once_cell::sync::Lazy;
use snips_nlu_lib::{ontology::IntentParserResult, SnipsNluEngine};

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
    Info { json: String },
    Notes,
    Corona,
    Reminder { json: String },
    Greet,
    About,
    Technology,
    Functions,
    Creator,
    Unknown,
}

fn parse(processed_text: &str) -> anyhow::Result<IntentParserResult> {
    Ok((&*NLUENGINE)
        .as_ref()
        .ok_or_else(|| anyhow!("NLU engine inactive"))?
        .parse_with_alternatives(
            processed_text,
            None,
            None,
            INTENTS_ALTERNATIVES,
            SLOTS_ALTERNATIVES,
        )
        .map_err(|err| anyhow!("Couldn't parse text {}", err))?)
}

///Uses natural understanding to determine intent if no state is found
pub async fn detect(processed_text: &str) -> anyhow::Result<Option<Intent>> {
    use Intent::{
        About, Animation, Chat, Corona, Creator, Functions, Greet, Identify, Info, Notes, Reminder,
        Search, Technology, Unknown,
    };

    let source = "NATURAL_ACTION_PICKER";

    let info = util::logger::info(source);
    let warning = util::logger::warning(source);
    let error = util::logger::error(source);

    //---Stuff required to run the NLU engine to get an intent
    if let Ok(result) = parse(processed_text) {
        if let Some(intent) = (&*NLUENGINE)
            .as_ref()
            .ok_or_else(|| anyhow!("NLU engine inactive"))?
            .parse_with_alternatives(
                processed_text,
                None,
                None,
                INTENTS_ALTERNATIVES,
                SLOTS_ALTERNATIVES,
            )
            .map_err(|err| anyhow!("Couldn't parse text {}", err))?
            .intent
            .intent_name
            .clone()
        {
            info(
                format!(
                    "{} with confidence {}",
                    intent, result.intent.confidence_score
                )
                .as_str(),
            );
            //Tries to match against existing intents like chat, search etc
            //Only valid if confidence greater than 0.5
            if result.intent.confidence_score > 0.5 {
                //---Convert result to json string
                if let Ok(json) = serde_json::to_string(&result) {
                    info("ACTION_PICKER: intent json is valid");
                    let intent_str: &str = intent.as_str();
                    info(format!("intent is {}", intent_str).as_str());
                    return match intent_str {
                        "chat" => Ok(Some(Chat)),
                        "search" => Ok(Some(Search)),
                        "identify" => Ok(Some(Identify)),
                        "animation" => Ok(Some(Animation)),
                        "info" => Ok(Some(Info { json })),
                        "notes" => Ok(Some(Notes)),
                        "corona" => Ok(Some(Corona)),
                        "reminder" => Ok(Some(Reminder { json })),
                        "unknown" => Ok(Some(Unknown)),
                        "greet" => Ok(Some(Greet)),
                        "about" => Ok(Some(About)),
                        "technology" => Ok(Some(Technology)),
                        "functions" => Ok(Some(Functions)),
                        "creator" => Ok(Some(Creator)),
                        _ => Ok(None),
                    };
                }
                //If failed to parse the intent result as json
                else {
                    error("couldn't convert intent data to JSON");
                    let _ = util::logger::log_message(processed_text)
                        .await
                        .map_err(|err| {
                            error(format!("{}", err).as_str());
                        });
                }
            }
            //Unsure intent if cannot match to any intent confidently
            else {
                warning("couldn't match an intent confidently");
                let _ = util::logger::log_message(processed_text)
                    .await
                    .map_err(|err| {
                        error(format!("{}", err).as_str());
                    });
            }
        }
        //Unknown intent if can't match intent at all
        else {
            warning("unknown intent");
            let _ = util::logger::log_message(processed_text)
                .await
                .map_err(|err| {
                    error(format!("{}", err).as_str());
                });
        };
    }
    return Err(anyhow::anyhow!("Nlu failed"));
}
