mod dates;
mod download;
mod io;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::path::Path;
use std::time::Duration;

pub use crate::dates::get_all_dates;
pub use crate::io::{create_dir, get_existing_dates, get_folder_path};

pub async fn download_all_images(
    folder: &Path,
    dates: &[NaiveDate],
    job_count: usize,
    attempt_count: u32,
    request_timeout: Duration,
) -> Result<(), String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.5666.197 Safari/537.36")
        .timeout(request_timeout)
        .build()
        .expect("Failed to build request client. This error should never occur.");

    let bodies =
        stream::iter(dates.into_iter().enumerate())
            .map(|(i, date)| {
                let job_id = i % job_count;
                let client = &client;
                async move {
                    download::download_image(client, *date, folder, job_id, attempt_count).await
                }
            })
            .buffer_unordered(job_count);

    bodies
        .for_each(|result| async {
            if let Err(err) = result {
                eprintln!("Error: {}", err);
            }
        })
        .await;

    Ok(())
}
