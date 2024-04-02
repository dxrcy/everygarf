use chrono::NaiveDate;
use reqwest::Client;

use crate::SourceAPI;

//TODO Inline this???
pub fn webpage_proxied(date: NaiveDate, proxy: Option<&str>, api: SourceAPI) -> String {
    let url = api.get_page_url(date);
    match proxy {
        None => url,
        Some(proxy) => proxy.to_string() + "?" + &url,
    }
}

pub async fn check_proxy_service(client: &Client, proxy: &str) -> Result<(), reqwest::Error> {
    client.get(proxy).send().await?.error_for_status()?;
    Ok(())
}
