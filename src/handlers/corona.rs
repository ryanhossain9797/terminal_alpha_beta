use crate::handlers::*;
use serde_json::Value;

#[allow(dead_code)]
struct Country {
    country: String,
    total_confirmed: String,
    total_deaths: String,
    new_confirmed: String,
    new_deaths: String,
}

#[allow(dead_code)]
struct Summary {}

pub async fn start_corona(m: Box<dyn root::BotMessage + Send + Sync>) {
    let url = "https://api.covid19api.com/summary".to_string();
    // let countries: Vec<Country> = vec![];
    match util::get_request_json(&url).await {
        Some(json_string) => {
            // println!("CORONA: json string is {}", json_string);
            match serde_json::from_str(&json_string).ok() {
                Some(Value::Object(map)) => {
                    match map.get("Global") {
                        Some(Value::Object(summary)) => {
                            println!("CORONA: value for key 'Global' found");
                            let mut total_confirmed: Option<i64> = None;
                            let mut total_deaths: Option<i64> = None;
                            if let Some(Value::Number(num)) = summary.get("TotalConfirmed") {
                                if let Some(total_confirmed_value) = num.as_i64() {
                                    total_confirmed = Some(total_confirmed_value);
                                }
                            }
                            if let Some(Value::Number(num)) = summary.get("TotalDeaths") {
                                if let Some(total_deaths_value) = num.as_i64() {
                                    total_deaths = Some(total_deaths_value);
                                }
                            }
                            match (total_confirmed, total_deaths) {
                                (Some(confirmed), Some(deaths)) => {
                                    (*m).send_msg(root::MsgCount::MultiMsg(vec![
                                        root::Msg::Text(
                                            match responses::load_response("corona-header") {
                                                Some(response) => response,
                                                _ => responses::response_unavailable(),
                                            },
                                        ),
                                        root::Msg::Text(
                                            match responses::load_response("corona-body") {
                                                Some(response) => response
                                                    .replace(
                                                        "{confirmed}",
                                                        &format!("{}", confirmed),
                                                    )
                                                    .replace("{deaths}", &format!("{}", deaths)),
                                                _ => responses::response_unavailable(),
                                            },
                                        ),
                                        root::Msg::Text(
                                            match responses::load_response("corona-footer") {
                                                Some(response) => response,
                                                _ => responses::response_unavailable(),
                                            },
                                        ),
                                    ]));
                                    return;
                                }
                                _ => println!("CORONA: couldn't get confirmed and deaths"),
                            }
                        }
                        _ => println!("CORONA: No Value for 'Global' key"),
                    }
                    match map.get("Countries") {
                        Some(Value::Array(_)) => println!("CORONA: Got country list"),
                        _ => println!("CORONA:  No Value for 'Countries' key"),
                    }
                }
                _ => println!("CORONA: json initial body doesn't match structure"),
            }
        }
        _ => println!("CORONA: couldn't get data"),
    }
    (*m).send_msg(root::MsgCount::SingleMsg(root::Msg::Text(
        match responses::load_response("corona-fail") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}
