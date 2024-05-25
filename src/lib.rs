pub mod api;
pub mod colors;
pub mod dates;
pub mod errors;

mod cache;
mod download;
mod io;

#[cfg(test)]
mod tests;

pub use crate::errors::Error;
pub use crate::io::{create_target_dir, get_folder_path};

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::{fs, path::Path, process, time::Duration};

use crate::colors::*;
use crate::dates::date_from_filename;
use crate::{api::Api, cache::DateUrlCached};

pub const PROXY_DEFAULT: &str = "https://proxy.darcy-700.workers.dev/cors-proxy";
pub const CACHE_DEFAULT: &str =
    "https://raw.githubusercontent.com/dxrcy/everygarf-cache/master/cache";

const ISSUE_URL: &str = "https://github.com/dxrcy/everygarf/issues/new";

pub const QUERY_SOME_EXITCODE: i32 = 10;

const MIN_COUNT_FOR_PING: usize = 10;

static mut PROGRESS_COUNT: u32 = 0;

pub fn get_existing_dates(folder: &Path) -> Result<Vec<NaiveDate>, String> {
    Ok(crate::io::get_child_filenames(folder)
        .map_err(|err| format!("read directory - {:#?}", err))?
        .filter_map(|filename| date_from_filename(filename.to_str()?))
        .collect())
}

#[derive(Clone, Copy)]
pub struct DownloadOptions<'a> {
    pub attempt_count: u32,
    pub api: Api<'a>,
    pub cache_file: Option<&'a str>,
    pub image_format: &'a str,
}

pub async fn download_all_images<'a>(
    folder: &Path,
    dates: &[NaiveDate],
    job_count: usize,
    [timeout_main, timeout_initial]: [Duration; 2],
    notify_on_fail: bool,
    cache_url: Option<String>,
    download_options: DownloadOptions<'a>,
    always_ping: bool,
) {
    let DownloadOptions {
        api, cache_file, ..
    } = download_options;

    const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36";

    let client_initial = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(timeout_initial)
        .build()
        .expect("Failed to build request client (initial). This error should never occur.");

    let client_main = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(timeout_main)
        .build()
        .expect("Failed to build request client (main). This error should never occur.");

    if let Some(proxy) = api.proxy {
        if !always_ping && dates.len() < MIN_COUNT_FOR_PING {
            println!("    {DIM}(Skipping proxy ping){RESET}");
        } else {
            println!("    {DIM}Pinging proxy server...{RESET}");
            if let Err(error) = api::check_proxy_service(&client_initial, proxy).await {
                let message = format!(
                "{RED}{BOLD}Proxy service unavailable{RESET} - {}.\n{DIM}Trying to ping {UNDERLINE}{}{RESET}\nPlease try later, or create an issue at {ISSUE_URL}",
                proxy,
                format_request_error(error),
            );
                fatal_error(Error::ProxyPing, message, notify_on_fail);
            }
        }
    }

    let dates_cached: Vec<_> = match cache_url {
        Some(cache_url) => {
            if cache::is_remote_url(&cache_url) {
                println!("    {DIM}Downloading cached URLs...{RESET}");
            } else {
                println!("    {DIM}Reading cached URLs...{RESET}");
            }
            let cached_dates = match cache::fetch_cached_urls(&client_initial, &cache_url).await {
                Ok(dates) => dates,
                Err(error) => {
                    let message = format!(
                        "{}\n{RESET}{DIM}Please try running with `--no-cache` argument, or create an issue at {ISSUE_URL}{RESET}",
                        error,
                    );
                    fatal_error(Error::CacheDownload, message, notify_on_fail)
                }
            };
            dates
                .iter()
                .map(|date| DateUrlCached {
                    date: *date,
                    url: cached_dates.get(date).cloned(),
                })
                .collect()
        }
        None => dates
            .iter()
            .map(|date| DateUrlCached {
                date: *date,
                url: None,
            })
            .collect(),
    };

    unsafe { PROGRESS_COUNT = 0 }

    let bodies = stream::iter(dates_cached.iter().enumerate())
        .map(|(i, date_cached)| {
            let job_id = i % job_count;
            let client = &client_main;
            let progress = dates_cached.len();
            async move {
                download::download_image(
                    client,
                    date_cached.clone(),
                    folder,
                    job_id,
                    progress,
                    download_options,
                )
                .await
            }
        })
        .buffered(job_count);

    bodies
        .for_each(|result| async {
            if let Err(error) = result {
                fatal_error(Error::DownloadFail, error, notify_on_fail);
            }
        })
        .await;

    if let Some(cache_file) = cache_file {
        if let Err(error) = cache::clean_cache_file(cache_file) {
            fatal_error(
                Error::CleanCache,
                format!("Failed to clean cache file - {}", error),
                notify_on_fail,
            );
        }
    }
}

pub fn fatal_error(code: Error, message: String, notify: bool) -> ! {
    eprintln!("{RED}=============[ERROR]============={RESET}");
    eprintln!("{YELLOW}{}", message);
    eprintln!("{RED}================================={RESET}");
    if notify {
        send_notification(&message);
    }
    process::exit(code as i32);
}

fn send_notification(message: &str) {
    let message = colors::remove_colors(message);
    notify_rust::Notification::new()
        .summary("EveryGarf Failed")
        .body(&format!("Download failed.\n{}", message))
        .timeout(Duration::from_secs(15))
        .show()
        .expect("Failed to show notification");
}

fn format_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        return format!("{YELLOW}Request timed out.{RESET} If this happens often, check your connection, or change the `--timeout` argument." );
    }
    if error.is_connect() {
        return format!("{YELLOW}Bad connection.{RESET} Check your internet access.");
    }

    let Some(errors) = error.status() else {
        return format!("{MAGENTA}Unknown error:{RESET} {:#?}", error);
    };
    let code = errors.as_u16();

    let message = match (errors, code) {
        (StatusCode::TOO_MANY_REQUESTS, _) => {
            format!("{RED}Rate limited.{RESET} Try again in a few minutes. See https://github.com/dxrcy/everygarf#proxy-service for more information.")
        }
        (_, 525) => "SSL handshake failed with Cloudflare.".to_string(),
        (_, 500) => "Server error - Try again later.".to_string(),
        _ => return format!("Uncommon error: {YELLOW}{}{RESET}", error),
    };

    format!("{YELLOW}{BOLD}{}{RESET} {}", code, message)
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1000 {
        return format!("{}B", bytes);
    }

    let mut bytes = bytes as f32 / 1000.0;
    let mut magnitude = 0;

    while bytes >= 1000.0 {
        bytes /= 1000.0;
        magnitude += 1;
    }

    let suffix = match magnitude {
        0 => "kB",
        1 => "MB",
        2 => "GB",
        3 => "TB",
        _ => unimplemented!("This is too many bytes. Something else has clearly gone wrong."),
    };

    format!("{:.1}{}", bytes, suffix)
}

pub fn format_duration(duration: Duration) -> String {
    let mut total = duration.as_secs();

    let seconds = total % 60;
    total /= 60;
    let minutes = total % 60;
    total /= 60;
    let hours = total % 24;
    total /= 24;
    let days = total;

    let mut output = String::new();
    let values = [(days, "d"), (hours, "h"), (minutes, "m"), (seconds, "s")];

    for (value, unit) in values {
        if value == 0 {
            continue;
        }

        if !output.is_empty() {
            output.push(' ');
        }

        output += &value.to_string();
        output += unit;
    }

    if output.is_empty() {
        return "0s".to_string();
    }

    output
}

pub fn get_dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0;

    let dir = fs::read_dir(path)?;
    for child in dir.flatten() {
        let path = child.path();

        if path.is_dir() {
            total_size += get_dir_size(path.as_path())?;
            continue;
        }

        let metadata = fs::metadata(&path)?;
        total_size += metadata.len();
    }

    Ok(total_size)
}
