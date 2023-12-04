use chrono::NaiveDate;
use reqwest::Client;

use crate::dates::date_to_string;

pub const PROXY_DEFAULT: &str = "https://proxy.darcy-700.workers.dev/cors-proxy";
pub const CACHE_DEFAULT: &str =
    "https://raw.githubusercontent.com/darccyy/everygarf-cache/master/everygarf.cache";

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

pub async fn fetch_cached_urls(
    client: &Client,
    cache_url: &str,
) -> Result<Vec<(NaiveDate, String)>, reqwest::Error> {
    let text = client
        .get(cache_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    println!("{}", text);

    todo!()
}
