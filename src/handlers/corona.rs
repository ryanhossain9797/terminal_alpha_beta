use super::*;
use serde_json::Value;

#[allow(dead_code)]
struct Country {
    country: String,
    total_confirmed: i64,
    total_deaths: i64,
    new_confirmed: i64,
    new_deaths: i64,
}

#[allow(dead_code)]
struct Summary {}

pub async fn start_corona(m: Box<dyn BotMessage + Send + Sync>) {
    let url = "https://api.covid19api.com/summary".to_string();
    // let countries: Vec<Country> = vec![];

    // println!("CORONA: json string is {}", json_string);
    match general::get_request_json(&url).await {
        Some(Value::Object(map)) => {
            (*m).send_message(MsgCount::SingleMsg(Msg::Text(
                match responses::load_response("corona-header") {
                    Some(response) => response,
                    _ => responses::response_unavailable(),
                },
            )));

            match map.get("Countries") {
                Some(Value::Array(country_list)) => {
                    println!("CORONA: Got country list");
                    let mut countries: Vec<Country> = vec![];
                    for country in country_list {
                        match (
                            country.get("Country"),
                            country.get("NewConfirmed"),
                            country.get("NewDeaths"),
                            country.get("TotalConfirmed"),
                            country.get("TotalDeaths"),
                        ) {
                            (
                                Some(Value::String(country)),
                                Some(Value::Number(new_confirmed_val)),
                                Some(Value::Number(new_deaths_val)),
                                Some(Value::Number(total_confirmed_val)),
                                Some(Value::Number(total_deaths_val)),
                            ) => match (
                                new_confirmed_val.as_i64(),
                                new_deaths_val.as_i64(),
                                total_confirmed_val.as_i64(),
                                total_deaths_val.as_i64(),
                            ) {
                                (
                                    Some(new_confirmed),
                                    Some(new_deaths),
                                    Some(total_confirmed),
                                    Some(total_deaths),
                                ) => countries.push(Country {
                                    country: country.clone(),
                                    new_confirmed,
                                    new_deaths,
                                    total_confirmed,
                                    total_deaths,
                                }),
                                _ => {
                                    println! {"CORONA: i64 conversion error"}
                                }
                            },
                            _ => {
                                println! {"CORONA: country keys error"}
                            }
                        }
                    }
                    countries.sort_unstable_by(|first, second| {
                        first.new_confirmed.cmp(&second.new_confirmed).reverse()
                    });
                    let mut new_cases_message = "Top new cases:\n".to_string();
                    for country in &countries[..10] {
                        new_cases_message += &format!(
                            "\nname: {}\nnew confirmed: {}\nnew deaths: {}\n",
                            country.country, country.new_confirmed, country.new_deaths
                        );
                    }
                    (*m).send_message(MsgCount::SingleMsg(Msg::Text(new_cases_message)));
                    countries.sort_unstable_by(|first, second| {
                        first.total_confirmed.cmp(&second.total_confirmed).reverse()
                    });
                    let mut total_cases_message = "Top total cases:\n".to_string();
                    for country in &countries[..10] {
                        total_cases_message += &format!(
                            "\nname: {}\ntotal confirmed: {}\ntotal deaths: {}\n",
                            country.country, country.total_confirmed, country.total_deaths
                        );
                    }
                    (*m).send_message(MsgCount::SingleMsg(Msg::Text(total_cases_message)));
                }
                _ => println!("CORONA:  No Value for 'Countries' key"),
            }
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
                            (*m).send_message(MsgCount::MultiMsg(vec![
                                Msg::Text(match responses::load_response("corona-body") {
                                    Some(response) => response
                                        .replace("{confirmed}", &format!("{}", confirmed))
                                        .replace("{deaths}", &format!("{}", deaths)),
                                    _ => responses::response_unavailable(),
                                }),
                                Msg::Text(match responses::load_response("corona-footer") {
                                    Some(response) => response,
                                    _ => responses::response_unavailable(),
                                }),
                            ]));
                        }
                        _ => println!("CORONA: couldn't get confirmed and deaths"),
                    }
                }
                _ => println!("CORONA: No Value for 'Global' key"),
            }
            return;
        }
        _ => println!("CORONA: json initial body doesn't match structure"),
    }
    (*m).send_message(MsgCount::SingleMsg(Msg::Text(
        match responses::load_response("corona-fail") {
            Some(response) => response,
            _ => responses::response_unavailable(),
        },
    )));
}
