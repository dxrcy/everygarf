use bytes::Bytes;
use chrono::{Datelike, NaiveDate};
use image::DynamicImage;
use reqwest::Client;
use std::path::Path;
use std::path::PathBuf;

use crate::api::Api;
use crate::cache;
use crate::colors::*;
use crate::format_request_error;
use crate::DateUrlCached;
use crate::SingleDownloadOptions;
use crate::PROGRESS_COUNT;

fn print_step(date: NaiveDate, job_id: usize, step: u32, total_count: usize) {
    let alt = if step < 2 { CYAN } else { "" };
    let icon = if step == 3 { "✓" } else { " " };
    let step = format!(
        "{}{step}{DIM}{}{RESET}",
        " ".repeat(step.max(1) as usize - 1),
        "•".repeat(3 - step.min(3) as usize),
    );
    let progress = unsafe { PROGRESS_COUNT } as usize * 100 / total_count;

    println!(
        "    {BOLD}{date}{RESET}  {DIM}#{job_id:02}{RESET}  {CYAN}{progress:-2}%{RESET}  {BLUE}{alt}[{step}{BLUE}{alt}]{RESET}  {GREEN}{icon}{RESET}"
    );
}

pub async fn download_image<'a>(
    client: &Client,
    date_cached: DateUrlCached,
    folder: &Path,
    job_id: usize,
    total_count: usize,
    download_options: SingleDownloadOptions<'a>,
) -> Result<(), String> {
    let SingleDownloadOptions {
        attempt_count,
        api,
        cache_file,
        image_format,
        save_as_tree,
    } = download_options;
    let date = date_cached.date;

    let filepath = if save_as_tree {
        match create_month_dir(folder, date) {
            Ok(month_dir) => {
                let day = pad_two_digits(date.day()) + "." + image_format;
                month_dir.join(day)
            }
            Err(error) => {
                return Err(format!(
                    "{} Failed to create parent directory - {error}",
                    date_cached.date,
                ))
            }
        }
    } else {
        let filename = format!("{}.{}", date.format("%Y-%m-%d"), image_format);
        folder.join(filename)
    };

    for attempt_no in 1..=attempt_count {
        let result = fetch_image(client, &date_cached, job_id, total_count, api, cache_file).await;
        match result {
            Ok(image) => {
                if let Err(error) = image.save(filepath) {
                    return Err(format!("{} Failed to save image file - {error}", date,));
                }
                unsafe { PROGRESS_COUNT += 1 }
                break;
            }
            Err(error) => {
                eprintln!("{YELLOW}[warning] {DIM}[Attempt {attempt_no}]{RESET} {BOLD}{}{RESET} {DIM}#{job_id}{RESET} Failed: {error}", date);
                if attempt_no >= attempt_count {
                    return Err(format!(
                        "{RESET}{BOLD}{}{RESET} Failed after {BOLD}{attempt_count}{RESET} attempts: {error}",
                        date,
                    ));
                }
            }
        }
    }

    Ok(())
}

fn create_month_dir(parent: &Path, date: NaiveDate) -> std::io::Result<PathBuf> {
    let year = parent.join(pad_two_digits(date.year() as u32));
    create_dir_if_not_exists(&year)?;
    let month = year.join(pad_two_digits(date.month()));
    create_dir_if_not_exists(&month)?;
    Ok(month)
}

fn pad_two_digits(number: u32) -> String {
    if number < 10 {
        "0".to_owned() + &number.to_string()
    } else {
        number.to_string()
    }
}

fn create_dir_if_not_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

async fn fetch_image<'a>(
    client: &Client,
    date_cached: &DateUrlCached,
    job_id: usize,
    total_count: usize,
    api: Api<'a>,
    cache_file: Option<&str>,
) -> Result<DynamicImage, String> {
    let image_url = match &date_cached.url {
        Some(url) => url.to_owned(),
        None => {
            print_step(date_cached.date, job_id, 1, total_count);
            fetch_image_url_from_date(client, date_cached.date, api)
                .await
                .map_err(|error| format!("Fetching image url - {}", error))?
        }
    };

    if let Some(cache_file) = cache_file {
        cache::append_cache_file(date_cached.date, &image_url, cache_file)?;
    }

    print_step(date_cached.date, job_id, 2, total_count);
    let image_bytes = fetch_image_bytes_from_url(client, &image_url)
        .await
        .map_err(|error| format!("Fetching image bytes - {}", error))?;

    print_step(date_cached.date, job_id, 3, total_count);
    let image = image::load_from_memory(&image_bytes)
        .map_err(|error| format!("Parsing image - {}", error))?;

    Ok(image)
}

async fn fetch_image_url_from_date<'a>(
    client: &Client,
    date: NaiveDate,
    api: Api<'a>,
) -> Result<String, String> {
    let url = api.get_page_url(date);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(format_request_error)?
        .error_for_status()
        .map_err(format_request_error)?;

    let response_body = response.text().await.map_err(|error| {
        format!("Converting webpage body for image URL to text ({url}) - {error}")
    })?;

    let Some(image_url) = api.source.find_image_url(&response_body) else {
        return Err(format!("Cannot find image URL in webpage body ({url})"));
    };

    Ok(image_url.to_owned())
}

async fn fetch_image_bytes_from_url(client: &Client, url: &str) -> Result<Bytes, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(format_request_error)?
        .error_for_status()
        .map_err(format_request_error)?;

    let bytes = response.bytes().await.map_err(format_request_error)?;

    Ok(bytes)
}
