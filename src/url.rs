use std::collections::HashMap;
use std::fs;

use chrono::NaiveDate;
use futures::TryFutureExt;
use reqwest::Client;

use crate::colors::*;
use crate::dates::date_from_filename;
use crate::{dates::date_to_string, format_request_error};

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

type DateMap = HashMap<NaiveDate, String>;

pub async fn fetch_cached_urls(client: &Client, cache_url: &str) -> Result<DateMap, String> {
    let text = if cache_url.starts_with("http://") || cache_url.starts_with("https://") {
        fetch_text(client, cache_url)
            .map_err(|error|
                format!(
                    "{RED}{BOLD}Remote cache download unavailable{RESET} - {}.\n{DIM}Trying to fetch {UNDERLINE}{}{RESET}\nPlease try later, run with `--no-cache` argument, or create an issue at https://github.com/darccyy/everygarf/issues/new",
                    cache_url,
                  format_request_error(  error),
                ))
            .await?
    } else {
        fs::read_to_string(cache_url)
            .map_err(|error| format!("Reading local cache file - {}", error))?
    };
    parse_cached_urls(&text).map_err(|_error| format!("Failed to parse cache file"))
}

fn parse_cached_urls(file: &str) -> Result<DateMap, ()> {
    let mut rows = HashMap::new();
    for line in file.lines() {
        let index = line.find(' ').ok_or(())?;
        let (date_string, url) = line.split_at(index);
        let date = date_from_filename(date_string.trim()).ok_or(())?;
        let url = url.trim().to_string();
        rows.insert(date, url);
    }
    Ok(rows)
}

async fn fetch_text(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
}
