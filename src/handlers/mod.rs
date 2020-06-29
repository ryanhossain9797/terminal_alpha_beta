//----------------------mod.rs name is mandatory as it is the main file for the handlers module
//----------------------'handlers/mod.rs' means handlers module

mod actions;
mod responses;
mod state;
extern crate snips_nlu_lib;

use crate::functions::*;
use actions::*;
use state::userstate::*;

use async_trait::async_trait;
use once_cell::sync::Lazy;

use snips_nlu_lib::SnipsNluEngine;
use std::fs::*;
use std::sync::Arc;
use std::time::Duration;

///Long wait time, Used in runing system
const LONGWAIT: u64 = 30;
#[allow(dead_code)]
///Short wait time, Used usually for testing
const SHORTWAIT: u64 = 10;

///Currently set waitime
const WAITTIME: u64 = LONGWAIT;

///NLUENGINE: Snips NLU is used to pick actions when they don't match directly
static NLUENGINE: Lazy<Option<SnipsNluEngine>> = Lazy::new(|| {
    println!("\nLoading the nlu engine...");
    SnipsNluEngine::from_path("data/rootengine/").ok()
});

///RESPONSES: Response json holding all the responses.  
///Put in a json so they can be modified without recompiling the bot.  
///Loaded at startup, Restart Bot to reload.
static RESPONSES: Lazy<Option<serde_json::Value>> = Lazy::new(|| {
    println!("\nLoading JSON responses");
    match read_to_string("data/responses.json") {
        Ok(json) => serde_json::from_str(&json).ok(),
        Err(_) => None,
    }
});

///HTTP client for..... HTTP things
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    println!("\nLoading Api Client");
    reqwest::Client::new()
});

///Initializes a variety of things
///- State management system
///- NLU engine
///- Responses JSON
pub fn initialize() {
    initialize_state();
    Lazy::force(&NLUENGINE);
    Lazy::force(&RESPONSES);
    Lazy::force(&CLIENT);
}

///ENUM, Represents Message count
///- SingleMsg - Contains a Msg Enum
///- MultiMsg - Contains a Vector of Msg Enums
///- NoMsg - Represnts an empty response
pub enum MsgCount {
    SingleMsg(Msg),
    MultiMsg(Vec<Msg>),
    // NoMsg,
}

///ENUM, Represents Message type
///- Text - Contains String text
///- File - Contains String url for file
pub enum Msg {
    Text(String),
    File(String),
}

///# Used to generalize Message Updates for various platforms
///All clients sending message updates must implement this
///## functions
///- `fn get_name() -> String` Return user readable name
///- `fn get_id() -> String` Return unique id for user
///- `async fn send_message(message: MsgCount)` Sends message to user
///- `fn start_conversation() -> bool` Returns bool indicating whether to start a new conversation
#[async_trait]
pub trait BotMessage: Send + Sync {
    ///This is used to make cloneable box<T> version of itself.
    // fn dynamic_clone(&self) -> Box<dyn BotMessage>;
    ///Returns the user's user readable name. Not the same as id.
    fn get_name(&self) -> String;
    ///Returns the user's unique id. This is needed to uniquely identify users.
    fn get_id(&self) -> String;
    ///Used to send messages to the sender (user) of this message.
    async fn send_message(&self, message: MsgCount);
    ///Used to check whether a new conversation should be started.  
    ///Sometimes if the user is in a state, Bot will always respond.  
    ///However if not in a state, bot needs to know when it should or should not respond.  
    ///Ex. Won't respond if message is in a group and bot wasn't mentioned.
    fn start_conversation(&self) -> bool;
}

///Distributes incoming requests to separate threads
pub fn distributor(bot_message: impl BotMessage + 'static, processed_text: String) {
    let source = "DISTRIBUTOR";
    //Spawn a new task to handle the message
    tokio::spawn(async move { handler(bot_message, processed_text).await });
    util::log_info(source, "Handler Thread Spawned");
}

///First place to handle messages after distribution
async fn handler(bot_message: impl BotMessage + 'static, processed_text: String) {
    let source = "HANDLER";
    util::log_info(source, &format!("Processed text is {}", processed_text));

    //---If record from user exists (A Some(record)), some conversation is ongoing
    //---So will be replied regardless of groups or mentions and stuff ('will_respond' is ignored)
    if let Some(stored_record) = get_state(&bot_message.get_id()).await {
        let record = stored_record.clone();

        //---"cancel last will shut off the conversation"
        if processed_text == "cancel last" {
            cancel_history(bot_message).await;
        } else if let UserState::Search = record.state {
            util::log_info(source, "continuing search");
            search::continue_search(bot_message, processed_text.clone()).await;
        }
        //---"if state is identify"
        else if let UserState::Identify = record.state {
            util::log_info(source, "continuing identify");
            identify::continue_identify(bot_message, processed_text.clone()).await;
        }
        //---"if state is animatios"
        else if let UserState::Animation = record.state {
            util::log_info(source, "continuing animation");
            animation::continue_gif(bot_message, processed_text.clone()).await;
        }
        //---"if state is animatios"
        else if let UserState::Notes(data) = record.state {
            util::log_info(source, "continuing notes");
            notes::continue_notes(bot_message, processed_text.clone(), data).await;
        }
        //---"if state is unknown"
        else {
            util::log_info(source, "some unknown state");
            responses::unknown_state_notice(bot_message).await;
        }
    }
    //---if record from user doesn't exist, but is either IN A PRIVATE CHAT or MENTIONED IN A GROUP CHAT
    //---will start processing new info
    else if bot_message.start_conversation() {
        //---cancel last does nothing as there's nothing to cancel
        if processed_text == "cancel last" {
            bot_message
                .send_message(MsgCount::SingleMsg(Msg::Text(
                    responses::load_named("cancel-nothing").unwrap_or_else(responses::unavailable),
                )))
                .await;
        }
        //---hand over to the natural understanding system for advanced matching
        else {
            natural_understanding(bot_message, processed_text).await;
        }
    }
}

///Uses natural understanding to determine intent if no state is found
async fn natural_understanding(bot_message: impl BotMessage + 'static, processed_text: String) {
    let source = "NATURAL_ACTION_PICKER";
    //---Stuff required to run the NLU engine to get an intent
    if let Some(engine) = &*NLUENGINE {
        let intents_alternatives = 1;
        let slots_alternatives = 1;
        let result = engine
            .parse_with_alternatives(
                &processed_text,
                None,
                None,
                intents_alternatives,
                slots_alternatives,
            )
            .unwrap();

        if let Some(intent) = result.intent.intent_name.clone() {
            util::log_info(
                source,
                &format!(
                    "{} with confidence {}",
                    intent, result.intent.confidence_score
                ),
            );
            //Tries to match against existing intents like chat, search etc
            //Only valid if confidence greater than 0.5
            if result.intent.confidence_score > 0.5 {
                //---Convert result to json string
                if let Ok(json) = serde_json::to_string(&result) {
                    util::log_info(source, "ACTION_PICKER: intent json is valid");
                    match &*intent {
                        "chat" => {
                            util::log_info(source, "starting chat");
                            chat::start_chat(bot_message).await
                        }
                        "search" => {
                            util::log_info(source, "starting search");
                            search::start_search(bot_message).await
                        }
                        "identify" => {
                            util::log_info(source, "starting identify");
                            identify::start_identify(bot_message).await
                        }
                        "animation" => {
                            util::log_info(source, "starting animation");
                            animation::start_gif(bot_message).await
                        }
                        "info" => {
                            util::log_info(source, "starting info");
                            info::start_info(bot_message, json).await
                        }
                        "notes" => {
                            util::log_info(source, "starting notes");
                            notes::start_notes(bot_message).await
                        }
                        "corona" => {
                            util::log_info(source, "starting corona");
                            corona::start_corona(bot_message).await
                        }
                        "unknown" => {
                            util::log_info(source, "starting unknown state test");
                            extras::start_unknown(bot_message).await
                        }
                        _ => {
                            //Forward to chat for more intents
                            util::log_info(source, "forwarding to chat");
                            chat::continue_chat(bot_message, processed_text, &intent).await;
                        }
                    }
                }
                //If failed to parse the intent result as json
                else {
                    util::log_error(source, "coldn't convert intent data to JSON");
                    general::log_message(processed_text);
                    responses::unsupported_notice(bot_message).await
                }
            }
            //Unsure intent if cannot match to any intent confidently
            else {
                util::log_warning(source, "couldn't match an intent confidently");
                general::log_message(processed_text.clone());
                responses::unsupported_notice(bot_message).await
            }
        }
        //Unknown intent if can't match intent at all
        else {
            util::log_warning(source, "unknown intent");
            general::log_message(processed_text.clone());
            responses::unsupported_notice(bot_message).await
        };
    } else {
        util::log_error(source, "NLU engine load failed");
        responses::unsupported_notice(bot_message).await
    }
}

///Removes current history with a cancellation message.  
///Doesn't care about state.  
///Used with the cancel last command.
async fn cancel_history(bot_message: impl BotMessage + 'static) {
    remove_state(&bot_message.get_id()).await;
    bot_message
        .send_message(MsgCount::SingleMsg(Msg::Text(
            responses::load_named("cancel-state").unwrap_or_else(responses::unavailable),
        )))
        .await;
}

///Removes history after 30 seconds if it's not updated with a new time,  
///AND the history state matches the provided state.  
///Notice Message is provided to user.
fn wipe_history(bot_message: Arc<impl BotMessage + 'static>, state: UserState) {
    let source = "WIPE_HISTORY";
    tokio::spawn(async move {
        //Wait a specified amount of time before deleting user state
        tokio::time::delay_for(Duration::from_secs(WAITTIME)).await;
        if let Some(record) = get_state(&bot_message.get_id()).await {
            //If the current state matches pending deletion state
            if format!("{}", record.state) == format!("{}", state) {
                //If the current state is older than threshold wait time
                if record.last.elapsed() > Duration::from_secs(WAITTIME) {
                    remove_state(&bot_message.get_id()).await;
                    util::log_info(source, &format!("deleted state record '{}'", state));
                    bot_message
                        .send_message(MsgCount::SingleMsg(Msg::Text(
                            responses::load_named("delay-notice")
                                .unwrap_or_else(responses::unavailable),
                        )))
                        .await;
                //If the current state is not older than threshold wait time
                } else {
                    util::log_info(source, "aborted record delete due to recency");
                }
            //If the current state doesn't match pending deletion state
            } else {
                util::log_info(
                    source,
                    &format!(
                        "aborted record delete for '{}' because current state is '{}'",
                        state, record.state
                    ),
                );
            }
        //If user has no pending state
        } else {
            util::log_info(
                source,
                &format!(
                    "aborted record delete for '{}', there is no recorded state for '{}'",
                    state,
                    bot_message.get_id()
                ),
            )
        }
    });
}

///Immediately purges history IF provided state matches history state.  
///Used to remove history after state action is completed.  
///No notice provided.
fn immediate_purge_history(bot_message: Arc<impl BotMessage + 'static>, state: UserState) {
    let source = "PURGE_HISTORY";
    tokio::spawn(async move {
        if let Some(r) = get_state(&bot_message.get_id()).await {
            if r.state == state {
                remove_state(&bot_message.get_id()).await;
                util::log_info(source, &format!("deleted state record for {}", state));
            }
        }
    });
}

///Removes history after 30 seconds if it's not updated with a new time,  
///AND the history state matches the provided state.  
///Notice Message is provided to user.
fn wipe_history_new(bot_message: Arc<impl BotMessage + 'static>) {
    let source = "WIPE_HISTORY";
    if let None = get_state(&bot_message.get_id()).await {
        return;
    }
    tokio::spawn(async move {
        while let Some(record) = get_state(&bot_message.get_id()).await {
            //If the current state is older than threshold wait time
            let elapsed = record.last.elapsed();
            if elapsed > Duration::from_secs(WAITTIME) {
                remove_state(&bot_message.get_id()).await;
                util::log_info(source, &format!("deleted state record '{}'", state));
                bot_message
                    .send_message(MsgCount::SingleMsg(Msg::Text(
                        responses::load_named("delay-notice")
                            .unwrap_or_else(responses::unavailable),
                    )))
                    .await;
            //If the current state is not older than threshold wait time
            } else {
                tokio::time::delay_for(Duration::from_secs(WAITTIME - elapsed.as_secs())).await;
                util::log_info(source, "aborted record delete due to recency");
            }
        }
        util::log_info(
            source,
            &format!("aborted record delete for user '{}'", bot_message.get_id()),
        )
    });
}
