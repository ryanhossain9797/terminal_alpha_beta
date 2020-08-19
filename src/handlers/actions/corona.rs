use super::*;

pub async fn start_corona(bot_message: impl BotMessage) {
    let source = "CORONA";
    let error = util::logger::make_error(source);

    let top_new = covid_service::get_top_new().await;
    let top_total = covid_service::get_top_total().await;
    let aggregate = covid_service::get_aggreagte().await;

    if let (&Err(_), &Err(_), &(Err(_), Err(_))) = (&top_new, &top_total, &aggregate) {
        //If the whole shebang fails
        bot_message
            .send_message(responses::load("corona-fail").into())
            .await;
    }

    bot_message
        .send_message(responses::load("corona-header").into())
        .await;

    if let Ok(top_new) = top_new {
        let mut new_cases_message = responses::load("corona-new-header")
            .unwrap_or_else(|| "(Fallback) Top new cases:\n".to_string());
        let new_template = responses::load_text("corona-new").unwrap_or_else(|| {
            "(Fallback)\nname: {1}\nnew confirmed: {2}\nnew deaths: {3}\n".to_string()
        });
        new_cases_message = top_new.iter().fold(new_cases_message, |message, country| {
            message
                + &new_template
                    .replace("{1}", &country.country)
                    .replace("{2}", &format!("{}", country.new_confirmed))
                    .replace("{3}", &format!("{}", country.new_deaths))
        });
        bot_message.send_message(new_cases_message.into()).await;
    }
    if let Ok(top_total) = top_total {
        let mut total_cases_message = responses::load("corona-total-header")
            .unwrap_or_else(|| "(Fallback) Top total cases:\n".to_string());
        let total_template = responses::load_text("corona-total").unwrap_or_else(|| {
            "(Fallback)\nname: {1}\ntotal confirmed: {2}\ntotal deaths: {3}\n".to_string()
        });
        total_cases_message = (top_total)
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

    match aggregate {
        (Ok(confirmed), Ok(deaths)) => {
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
