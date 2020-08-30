use super::*;
use async_std::sync::{Mutex, MutexGuard};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::time::*;

#[derive(Clone)]
pub struct Country {
    pub country: String,
    pub total_confirmed: i64,
    pub total_deaths: i64,
    pub new_confirmed: i64,
    pub new_deaths: i64,
}

#[derive(Clone)]
pub struct Covid {
    time: Instant,
    top_new: Option<Vec<Country>>,
    top_total: Option<Vec<Country>>,
    aggregated_total_confirmed: Option<i64>,
    aggregated_total_deaths: Option<i64>,
}

static CACHE: Lazy<Mutex<Covid>> = Lazy::new(|| Mutex::new(Covid::new()));

impl Covid {
    fn new() -> Self {
        Self {
            time: Instant::now() - Duration::from_secs(1200),
            top_new: None,
            top_total: None,
            aggregated_total_confirmed: None,
            aggregated_total_deaths: None,
        }
    }
    async fn update_cache(&mut self) {
        let source = "COVID_SERVICE";
        let error = util::logger::error(source);
        let info = util::logger::info(source);

        let url = "https://api.covid19api.com/summary".to_string();
        match api::get_request_json(&url).await {
            //If Successful
            Ok(Value::Object(map)) => {
                //Work through the json to get the country specific data
                match map.get("Countries") {
                    Some(Value::Array(country_list)) => {
                        info("Got country list");
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
                        self.top_new = countries[0..10].to_vec().into();

                        countries.sort_unstable_by(|first, second| {
                            first.total_confirmed.cmp(&second.total_confirmed).reverse()
                        });
                        self.top_total = countries[0..10].to_vec().into();
                    }
                    _ => error("No Value for 'Countries' key"),
                }
                match map.get("Global") {
                    Some(Value::Object(summary)) => {
                        info("Value for key 'Global' found");

                        if let Some(Value::Number(num)) = summary.get("TotalConfirmed") {
                            self.aggregated_total_confirmed = num.as_i64();
                        }
                        if let Some(Value::Number(num)) = summary.get("TotalDeaths") {
                            self.aggregated_total_deaths = num.as_i64();
                        }
                    }
                    _ => error("No Value for 'Global' key"),
                }
            }
            Ok(_) => error("Fetched JSON doesn't match strcuture"),
            Err(err) => error(&format!("Failed to fetch JSON response: {}", err)),
        }
        self.time = Instant::now();
    }

    async fn refresh(&mut self) {
        if self.time.elapsed() > Duration::from_secs(600) {
            let info = util::logger::info("COVID_CACHE");
            info("Refreshing cache");
            self.update_cache().await;
        }
    }

    pub async fn get_top_new(&mut self) -> anyhow::Result<Vec<Country>> {
        self.refresh().await;
        self.top_new
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Couldn't fetch top new"))
    }

    pub async fn get_top_total(&mut self) -> anyhow::Result<Vec<Country>> {
        self.refresh().await;
        self.top_total
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Couldn't fetch top total"))
    }

    pub async fn get_aggregate(&mut self) -> (anyhow::Result<i64>, anyhow::Result<i64>) {
        self.refresh().await;
        (
            self.aggregated_total_confirmed
                .ok_or_else(|| anyhow::anyhow!("Couldn't fetch aggregated total confirmed")),
            self.aggregated_total_deaths
                .ok_or_else(|| anyhow::anyhow!("Couldn't fetch aggregated total deaths")),
        )
    }
}

pub async fn get() -> MutexGuard<'static, Covid> {
    CACHE.lock().await
}
