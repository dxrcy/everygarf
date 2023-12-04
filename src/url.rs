use crate::dates::date_to_string;
use chrono::NaiveDate;
use reqwest::Client;

pub fn webpage_proxied(date: NaiveDate, proxy: Option<&str>) -> String {
    let url = webpage_unproxied(date);
    match proxy {
        None => url,
        Some(proxy) => proxy.to_string() + "?" + &url,
    }
}

fn webpage_unproxied(date: NaiveDate) -> String {
    let date_string = date_to_string(date, "/", false);
    format!("https://www.gocomics.com/garfield/{}", date_string)
}

pub async fn check_proxy_service(client: &Client, proxy: &str) -> Result<(), reqwest::Error> {
    client.get(proxy).send().await?.error_for_status()?;
    Ok(())
}
