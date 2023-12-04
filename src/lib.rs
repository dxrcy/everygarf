mod cache;
pub mod colors;
pub mod dates;
mod download;
mod io;
pub mod proxy;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::{path::Path, process, time::Duration};

use crate::cache::DateUrlCached;
use crate::colors::*;
use crate::dates::date_from_filename;
pub use crate::io::{create_target_dir, get_folder_path};

pub const PROXY_DEFAULT: &str = "https://proxy.darcy-700.workers.dev/cors-proxy";
pub const CACHE_DEFAULT: &str =
    "https://raw.githubusercontent.com/darccyy/everygarf-cache/master/cache";

pub fn get_existing_dates(folder: &Path) -> Result<Vec<NaiveDate>, String> {
    Ok(crate::io::get_child_filenames(folder)
        .map_err(|err| format!("read directory - {:#?}", err))?
        .filter_map(|filename| date_from_filename(filename.to_str()?))
        .collect())
}

pub async fn download_all_images(
    folder: &Path,
    dates: &[NaiveDate],
    job_count: usize,
    attempt_count: u32,
    request_timeout: Duration,
    notify_fail: bool,
    proxy: Option<String>,
    cache_url: Option<String>,
    cache_file: Option<String>,
) {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36")
        .timeout(request_timeout)
        .build()
        .expect("Failed to build request client. This error should never occur.");

    let proxy = proxy.as_deref();
    if let Some(proxy) = proxy {
        println!("    {DIM}Pinging proxy server...{RESET}");
        if let Err(error) = proxy::check_proxy_service(&client, proxy).await {
            let message = format!(
                "{RED}{BOLD}Proxy service unavailable{RESET} - {}.\n{DIM}Trying to ping {UNDERLINE}{}{RESET}\nPlease try later, or create an issue at https://github.com/darccyy/everygarf/issues/new",
                proxy,
                format_request_error(error),
            );
            fatal_error(4, message, notify_fail);
        }
    }

    let dates_cached: Vec<_> = match cache_url {
        Some(cache_url) => {
            println!("    {DIM}Downloading cached URLs...{RESET}");
            let cached_dates = match cache::fetch_cached_urls(&client, &cache_url).await {
                Ok(dates) => dates,
                Err(error) => {
                    let message = format!(
                        "{}\n{RESET}{DIM}Please try running with `--no-cache` argument, or create an issue at https://github.com/darccyy/everygarf/issues/new{RESET}",
                        error,
                    );
                    fatal_error(5, message, notify_fail)
                }
            };
            dates
                .into_iter()
                .map(|date| DateUrlCached {
                    date: *date,
                    url: cached_dates.get(date).cloned(),
                })
                .collect()
        }
        None => dates
            .into_iter()
            .map(|date| DateUrlCached {
                date: *date,
                url: None,
            })
            .collect(),
    };
    let cache_file = cache_file.as_deref();

    let min_count_for_progress = job_count * 20;

    let bodies = stream::iter(dates_cached.iter().enumerate())
        .map(|(i, date_cached)| {
            let job_id = i % job_count;
            let client = &client;
            let progress = if dates_cached.len() >= min_count_for_progress {
                Some(i * 100 / dates_cached.len())
            } else {
                None
            };
            async move {
                download::download_image(
                    client,
                    date_cached.clone(),
                    folder,
                    job_id,
                    attempt_count,
                    progress,
                    proxy,
                    cache_file,
                )
                .await
            }
        })
        .buffered(job_count);

    bodies
        .for_each(|result| async {
            if let Err(error) = result {
                fatal_error(1, error, notify_fail);
            }
        })
        .await;

    if let Some(cache_file) = cache_file {
        if let Err(error) = cache::clean_cache_file(&cache_file) {
            fatal_error(
                6,
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

    let Some(status) = error.status() else {
        return format!("{MAGENTA}Unknown error:{RESET} {:#?}", error);
    };
    let code = status.as_u16();

    let message = match (status, code) {
        (StatusCode::TOO_MANY_REQUESTS, _) => {
            format!("{RED}Rate limited.{RESET} Try again in a few minutes. See https://github.com/darccyy/everygarf#proxy-service for more information.")
        }
        (_, 525) => "SSL handshake failed with Cloudflare.".to_string(),
        (_, 500) => "Server error - Try again later.".to_string(),
        _ => return format!("Uncommon error: {YELLOW}{}{RESET}", error),
    };

    format!("{YELLOW}{BOLD}{}{RESET} {}", code, message)
}
