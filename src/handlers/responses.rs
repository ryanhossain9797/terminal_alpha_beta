//
extern crate snips_nlu_lib;
use snips_nlu_lib::SnipsNluEngine;
//

use telegram_bot::*;

lazy_static! {
    pub static ref ENGINE: SnipsNluEngine = {
        println!("\nLoading the nlu engine...");
        SnipsNluEngine::from_path("engine/").unwrap()
    };
}

pub async fn unsupported_notice(api: Api, message: Message) -> Result<(), Error> {
    let notice_result = api
        .send(message.chat.text(format!(
            "we could not understand that\
                \nplease be aware that we are a test system with only sub-functions available\
                \nwe can only utilize a fraction of our full capabilites on this server"
        )))
        .await;
    match notice_result {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(())
}

pub async fn natural_understanding(
    api: Api,
    message: Message,
    processed_text: String,
) -> Result<(), Error> {
    let intents_alternatives = 1;
    let slots_alternatives = 1;

    let result = ENGINE
        .parse_with_alternatives(
            &processed_text,
            None,
            None,
            intents_alternatives,
            slots_alternatives,
        )
        .unwrap();
    if let Some(intent) = result.intent.intent_name {
        println!(
            "{} with confidence {}",
            intent, result.intent.confidence_score
        );
        if result.intent.confidence_score > 0.5 {
            if intent == "MakeTea" {
                let notice_result = api
                    .send(message.chat.text(format!(
                        "you want us to make tea?\
                \nwe do not know how to make human beverages"
                    )))
                    .await;
                match notice_result {
                    Err(e) => println!("{:?}", e),
                    _ => (),
                }
            } else if intent == "MakeCoffee" {
                let notice_result = api
                    .send(message.chat.text(format!(
                        "you want us to make coffee for you?\
            \nwe do not know how to make human beverages"
                    )))
                    .await;
                match notice_result {
                    Err(e) => println!("{:?}", e),
                    _ => (),
                }
            }
        } else {
            println!("unknown intent");
            let handler_assignment = unsupported_notice(api.clone(), message.clone()).await;
            match handler_assignment {
                Err(e) => println!("{:?}", e),
                _ => (),
            }
        }
    } else {
        println!("could not understand intent");
        let handler_assignment = unsupported_notice(api.clone(), message.clone()).await;
        match handler_assignment {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    Ok(())
}
