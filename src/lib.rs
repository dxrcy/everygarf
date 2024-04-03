pub mod colors;
pub mod dates;
pub mod errors;

pub mod api;
mod cache;
mod download;
mod io;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::{path::Path, process, time::Duration};

use crate::colors::*;
use crate::dates::date_from_filename;
pub use crate::io::{create_target_dir, get_folder_path};
use crate::{api::SourceApi, cache::DateUrlCached};

pub const PROXY_DEFAULT: &str = "https://proxy.darcy-700.workers.dev/cors-proxy";
pub const CACHE_DEFAULT: &str =
    "https://raw.githubusercontent.com/dxrcy/everygarf-cache/master/cache";

const ISSUE_URL: &str = "https://github.com/dxrcy/everygarf/issues/new";

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
    pub source_api: SourceApi<'a>,
    pub cache_file: Option<&'a str>,
    pub image_format: &'a str,
}

pub async fn download_all_images<'a>(
    folder: &Path,
    dates: &[NaiveDate],
    job_count: usize,
    [timeout_main, timeout_initial]: [Duration; 2],
    notify_fail: bool,
    cache_url: Option<String>,
    download_options: DownloadOptions<'a>,
) {
    let DownloadOptions {
        source_api,
        cache_file,
        ..
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

    if let Some(proxy) = source_api.proxy {
        println!("    {DIM}Pinging proxy server...{RESET}");
        if let Err(error) = api::check_proxy_service(&client_initial, proxy).await {
            let message = format!(
                "{RED}{BOLD}Proxy service unavailable{RESET} - {}.\n{DIM}Trying to ping {UNDERLINE}{}{RESET}\nPlease try later, or create an issue at {ISSUE_URL}",
                proxy,
                format_request_error(error),
            );
            fatal_error(errors::PROXY_PING, message, notify_fail);
        }
    }

    dbg!(source_api);

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
                    fatal_error(errors::CACHE_DOWNLOAD, message, notify_fail)
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
                fatal_error(errors::DOWNLOAD_FAIL, error, notify_fail);
            }
        })
        .await;

    if let Some(cache_file) = cache_file {
        if let Err(error) = cache::clean_cache_file(cache_file) {
            fatal_error(
                errors::CLEAN_CACHE,
                format!("Failed to clean cache file - {}", error),
                notify_fail,
            );
        }
    }
}

pub fn fatal_error(code: u8, message: String, notify: bool) -> ! {
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
