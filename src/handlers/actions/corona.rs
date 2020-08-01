use super::*;
use serde_json::Value;

struct Country {
    country: String,
    total_confirmed: i64,
    total_deaths: i64,
    new_confirmed: i64,
    new_deaths: i64,
}

pub async fn start_corona(bot_message: impl BotMessage) {
    let source = "CORONA";
    let error = util_service::make_error(source);

    let top_new = covid_service::get_top_new();
    let top_total = covid_service::get_top_total();
    let aggregate = covid_service::get_aggreagte();

    let url = "https://api.covid19api.com/summary".to_string();

    //Fetch country data from API
    match util_service::get_request_json(&url).await {
        //If Successful
        Some(Value::Object(map)) => {
            bot_message
                .send_message(responses::load("corona-header").into())
                .await;

            //Work through the json to get the country specific data
            match map.get("Countries") {
                Some(Value::Array(country_list)) => {
                    println!("CORONA: Got country list");
                    let mut countries: Vec<Country> = vec![];
                    country_list.iter().for_each(|country| {
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
                                    error("i64 conversion error");
                                }
                            },
                            _ => {
                                error("country keys error");
                            }
                        }
                    });
                    countries.sort_unstable_by(|first, second| {
                        first.new_confirmed.cmp(&second.new_confirmed).reverse()
                    });
                    let mut new_cases_message = responses::load("corona-new-header")
                        .unwrap_or_else(|| "(Fallback) Top new cases:\n".to_string());
                    let new_template = responses::load_text("corona-new").unwrap_or_else(|| {
                        "(Fallback)\nname: {1}\nnew confirmed: {2}\nnew deaths: {3}\n".to_string()
                    });
                    new_cases_message =
                        (&countries[..10])
                            .iter()
                            .fold(new_cases_message, |message, country| {
                                message
                                    + &new_template
                                        .replace("{1}", &country.country)
                                        .replace("{2}", &format!("{}", country.new_confirmed))
                                        .replace("{3}", &format!("{}", country.new_deaths))
                            });
                    bot_message.send_message(new_cases_message.into()).await;
                    countries.sort_unstable_by(|first, second| {
                        first.total_confirmed.cmp(&second.total_confirmed).reverse()
                    });
                    let mut total_cases_message = responses::load("corona-total-header")
                        .unwrap_or_else(|| "(Fallback) Top total cases:\n".to_string());
                    let total_template =
                        responses::load_text("corona-total").unwrap_or_else(|| {
                            "(Fallback)\nname: {1}\ntotal confirmed: {2}\ntotal deaths: {3}\n"
                                .to_string()
                        });
                    total_cases_message =
                        (&countries[..10])
                            .iter()
                            .fold(total_cases_message, |message, country| {
                                message
                                    + &total_template
                                        .replace("{1}", &country.country)
                                        .replace("{2}", &format!("{}", country.total_confirmed))
                                        .replace("{3}", &format!("{}", country.total_deaths))
                            });
                    bot_message.send_message(total_cases_message.into()).await;
                }
                _ => error("No Value for 'Countries' key"),
            }

            //Work through the json to get global data
            match map.get("Global") {
                Some(Value::Object(summary)) => {
                    println!("CORONA: value for key 'Global' found");
                    let mut total_confirmed: Option<i64> = None;
                    let mut total_deaths: Option<i64> = None;
                    if let Some(Value::Number(num)) = summary.get("TotalConfirmed") {
                        total_confirmed = num.as_i64()
                    }
                    if let Some(Value::Number(num)) = summary.get("TotalDeaths") {
                        total_deaths = num.as_i64();
                    }
                    match (total_confirmed, total_deaths) {
                        (Some(confirmed), Some(deaths)) => {
                            bot_message
                                .send_message(MsgCount::MultiMsg(vec![
                                    (match responses::load("corona-body") {
                                        Some(response) => Some(
                                            response
                                                .replace("{confirmed}", &format!("{}", confirmed))
                                                .replace("{deaths}", &format!("{}", deaths)),
                                        ),
                                        _ => None,
                                    })
                                    .into(),
                                    responses::load("corona-footer").into(),
                                ]))
                                .await;
                        }
                        _ => error("couldn't get confirmed and deaths"),
                    }
                }
                _ => error("No Value for 'Global' key"),
            }
            return;
        }
        _ => error("json initial body doesn't match structure"),
    }
    //If the whole shebang fails
    bot_message
        .send_message(responses::load("corona-fail").into())
        .await;
}
