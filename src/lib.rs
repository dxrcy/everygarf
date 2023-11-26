pub mod colors;
mod dates;
mod download;
mod io;

use chrono::NaiveDate;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::{path::Path, time::Duration};

use crate::dates::date_from_filename;
pub use crate::dates::get_all_dates;
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
                std::process::exit(1);
            }
        })
        .await;

    Ok(())
}
