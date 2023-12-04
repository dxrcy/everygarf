use std::collections::HashMap;
use std::fs;

use chrono::NaiveDate;
use futures::{io, TryFutureExt};
use reqwest::Client;

use crate::colors::*;
use crate::dates::date_from_filename;
use crate::format_request_error;

type DateMap = HashMap<NaiveDate, String>;

pub async fn fetch_cached_urls(client: &Client, cache_url: &str) -> Result<DateMap, String> {
    let text = if is_remote_url(cache_url) {
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

fn is_remote_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn parse_cached_urls(file: &str) -> Result<DateMap, ()> {
    let mut rows = HashMap::new();
    for line in file.lines() {
        let (date_string, url) = split_first_word(line).ok_or(())?;
        let date = date_from_filename(date_string.trim()).ok_or(())?;
        let url = url.trim().to_string();
        rows.insert(date, url);
    }
    Ok(rows)
}

fn split_first_word(string: &str) -> Option<(&str, &str)> {
    let index = string.find(' ')?;
    Some(string.split_at(index))
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

pub fn clean_cache_file(cache_file: &str) -> io::Result<()> {
    let file = fs::read_to_string(cache_file)?;
    let rows: Vec<_> = file.lines().collect();

    // remove duplicates
    // keep last instance of each date
    let mut unique_rows = Vec::new();
    let mut seen_dates = Vec::new();
    for row in rows.into_iter().rev() {
        let Some((date, _)) = split_first_word(row) else {
            continue;
        };
        if seen_dates.contains(&date) {
            continue;
        }
        unique_rows.push(row);
        seen_dates.push(&date);
    }

    // sort by date (alphabetically)
    unique_rows.sort();

    let file = unique_rows.join("\n");
    fs::write(cache_file, file)
}
