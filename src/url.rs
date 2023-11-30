use chrono::NaiveDate;
use reqwest::Client;

use crate::colors::*;
use crate::dates::date_to_string;
use crate::format_request_error;

pub fn webpage_proxied(date: NaiveDate) -> String {
    with_proxy_service(&webpage_unproxied(date))
}

pub fn webpage_unproxied(date: NaiveDate) -> String {
    let date_string = date_to_string(date, "/", false);
    format!("https://www.gocomics.com/garfield/{}", date_string)
}

pub fn with_proxy_service(url: &str) -> String {
    format!("https://proxy.darcy-700.workers.dev/{}", url)
}

pub async fn check_proxy_service(client: &Client) -> Result<(), String> {
    let url = with_proxy_service("");
    let result = client
        .get(&url)
        .send()
        .await
        .and_then(|response| response.error_for_status());

    if let Err(error) = result {
        let message = format!(
            "{RED}{BOLD}Proxy service unavailable{RESET} - {}.\n{DIM}Trying to ping {UNDERLINE}{url}{RESET}\nPlease try later, or create an issue at https://github.com/darccyy/everygarf/issues/new",
            format_request_error(error),
        );
        return Err(message);
    }

    Ok(())
}
