mod dates;
mod download;
mod io;

use futures::{stream, StreamExt};
use reqwest::Client;
use std::path::Path;

pub use io::{create_empty_target_directory, get_folder_path};

pub async fn download_all_images(folder: &Path) -> Result<(), String> {
    let job_count = 20;

    let client = Client::new();
    let dates = dates::get_all_dates().into_iter().enumerate();

    let bodies = stream::iter(dates)
        .map(|(i, date)| {
            let job_id = i % job_count;
            let client = &client;
            async move { download::download_image(client, date, folder, job_id, 10).await }
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
