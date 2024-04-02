use chrono::{Datelike, NaiveDate};
use reqwest::Client;

use crate::dates::date_to_string;

pub fn webpage_proxied(date: NaiveDate, proxy: Option<&str>) -> String {
    let url = webpage_unproxied(date);
    match proxy {
        None => url,
        Some(proxy) => proxy.to_string() + "?" + &url,
    }
}

fn webpage_unproxied(date: NaiveDate) -> String {
    let month_string = date_month_to_string(date);
    let date_string = date_to_string(date, "-", false);
    format!(
        "https://garfield.fandom.com/wiki/Garfield,_{month}_{year}_comic_strips?file={date}.gif",
        month = month_string,
        year = date.year(),
        date = date_string,
    )
}

fn date_month_to_string(date: NaiveDate) -> &'static str {
    match date.month0() {
        0 => "January",
        1 => "February",
        2 => "March",
        3 => "April",
        4 => "May",
        5 => "June",
        6 => "July",
        7 => "August",
        8 => "September",
        9 => "October",
        10 => "November",
        11 => "December",
        _ => unreachable!(),
    }
}

pub async fn check_proxy_service(client: &Client, proxy: &str) -> Result<(), reqwest::Error> {
    client.get(proxy).send().await?.error_for_status()?;
    Ok(())
}
