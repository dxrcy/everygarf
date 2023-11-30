pub mod colors;
pub mod dates;
mod download;
mod io;
mod url;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use std::{path::Path, process, time::Duration};

use crate::colors::*;
use crate::dates::date_from_filename;
pub use crate::io::{create_target_dir, get_folder_path};

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
    use_proxy: bool,
) {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36")
        .timeout(request_timeout)
        .build()
        .expect("Failed to build request client. This error should never occur.");

    if use_proxy {
        if let Err(error) = url::check_proxy_service(&client).await {
            fatal_error(4, error, notify_fail);
        }
    }

    let bodies = stream::iter(dates.iter().enumerate())
        .map(|(i, date)| {
            let job_id = i % job_count;
            let client = &client;
            async move {
                download::download_image(client, *date, folder, job_id, attempt_count, use_proxy)
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
        .timeout(Duration::from_secs(10))
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
            format!("{RED}Rate limited.{RESET} Try again in a few minutes, or change IP.")
        }
        (_, 525) => "SSL handshake failed with Cloudflare.".to_string(),
        (_, 500) => "Server error - Try again later.".to_string(),
        _ => return format!("Uncommon error: {YELLOW}{}{RESET}", error),
    };

    format!("{YELLOW}{BOLD}{}{RESET} {}", code, message)
}
