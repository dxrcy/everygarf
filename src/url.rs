use chrono::NaiveDate;
use reqwest::Client;

use crate::colors::*;
use crate::dates::date_to_string;
use crate::format_request_error;

pub const PROXY_DEFAULT: &str = "https://proxy.darcy-700.workers.dev/cors-proxy";

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

pub async fn check_proxy_service(client: &Client, proxy: &str) -> Result<(), String> {
    let result = client
        .get(proxy)
        .send()
        .await
        .and_then(|response| response.error_for_status());

    if let Err(error) = result {
        let message = format!(
            "{RED}{BOLD}Proxy service unavailable{RESET} - {}.\n{DIM}Trying to ping {UNDERLINE}{}{RESET}\nPlease try later, or create an issue at https://github.com/darccyy/everygarf/issues/new",
            proxy,
            format_request_error(error),
        );
        return Err(message);
    }

    Ok(())
}
