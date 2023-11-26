mod dates;
mod download;

use futures::{stream, StreamExt};
use reqwest::Client;
use std::{fs, path::Path};

pub fn create_empty_target_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Err(format!("target directory is a file"));
        }
        fs::remove_dir_all(path).map_err(|err| format!("remove directory - {:#?}", err))?;
    }
    fs::create_dir_all(path).map_err(|err| format!("create directory - {:#?}", err))?;
    Ok(())
}

pub async fn download_all_images(folder: &Path) -> Result<(), String> {
    let job_count = 20;

    let client = Client::new();
    let dates = dates::get_all_dates().into_iter().enumerate();

    let bodies = stream::iter(dates)
        .map(|(i, date)| {
            let client = &client;
            async move { download::download_image(client, date, folder, i, 10).await }
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
