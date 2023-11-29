pub mod colors;
pub mod dates;
mod download;
mod io;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::Client;
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
    notify_error: bool,
) {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36")
        .timeout(request_timeout)
        .build()
        .expect("Failed to build request client. This error should never occur.");

    let bodies =
        stream::iter(dates.iter().enumerate())
            .map(|(i, date)| {
                let job_id = i % job_count;
                let client = &client;
                async move {
                    download::download_image(client, *date, folder, job_id, attempt_count).await
                }
            })
            .buffered(job_count);

    bodies
        .for_each(|result| async {
            if let Err(err) = result {
                fatal_error(1, err, notify_error);
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
