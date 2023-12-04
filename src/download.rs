use bytes::Bytes;
use chrono::NaiveDate;
use image::DynamicImage;
use reqwest::Client;
use std::{fs, io::Write, path::Path};

use crate::colors::*;
use crate::dates::date_to_string;
use crate::format_request_error;
use crate::proxy;
use crate::DateUrlCached;

fn print_step(date: NaiveDate, job_id: usize, step: u32) {
    let alt = if step < 2 {
        format!("{CYAN}")
    } else {
        String::new()
    };
    let icon = if step == 3 { "✓" } else { " " };
    let step = format!(
        "{}{step}{DIM}{}{RESET}",
        " ".repeat(step.max(1) as usize - 1),
        "•".repeat(3 - step.min(3) as usize),
    );
    println!(
        "    {BOLD}{date}{RESET}  {DIM}#{job_id:02}{RESET}  {BLUE}{alt}[{step}{BLUE}{alt}]{RESET}  {GREEN}{icon}{RESET}"
    );
}

pub async fn download_image(
    client: &Client,
    date_cached: DateUrlCached,
    folder: &Path,
    job_id: usize,
    attempt_count: u32,
    proxy: Option<&str>,
    cache_file: Option<&str>,
) -> Result<(), String> {
    let filename = date_to_string(date_cached.date, "-", true) + ".png";
    let filename = Path::new(&filename);
    let filepath = folder.join(filename);

    for attempt_no in 1..=attempt_count {
        let result = fetch_image(client, &date_cached, job_id, proxy, cache_file).await;
        match result {
            Ok(image) => {
                if let Err(error) = image.save(filepath) {
                    return Err(format!(
                        "{} Failed to save image file - {error}",
                        date_cached.date,
                    ));
                }
                break;
            }
            Err(error) => {
                eprintln!("{YELLOW}[warning] {DIM}[Attempt {attempt_no}]{RESET} {BOLD}{}{RESET} {DIM}#{job_id}{RESET} Failed: {error}", date_cached.date);
                if attempt_no >= attempt_count {
                    return Err(format!(
                        "{RESET}{BOLD}{}{RESET} Failed after {BOLD}{attempt_count}{RESET} attempts: {error}",
                        date_cached.date,
                    ));
                }
            }
        }
    }

    Ok(())
}

async fn fetch_image(
    client: &Client,
    date_cached: &DateUrlCached,
    job_id: usize,
    proxy: Option<&str>,
    cache_file: Option<&str>,
) -> Result<DynamicImage, String> {
    let image_url = match &date_cached.url {
        Some(url) => url.to_owned(),
        None => {
            print_step(date_cached.date, job_id, 1);
            fetch_image_url_from_date(client, date_cached.date, proxy)
                .await
                .map_err(|error| format!("Fetching image url - {}", error))?
        }
    };

    if let Some(cache_file) = cache_file {
        append_cache_file(date_cached.date, &image_url, cache_file)?;
    }

    print_step(date_cached.date, job_id, 2);
    let image_bytes = fetch_image_bytes_from_url(client, &image_url)
        .await
        .map_err(|error| format!("Fetching image bytes - {}", error))?;

    print_step(date_cached.date, job_id, 3);
    let image = image::load_from_memory(&image_bytes)
        .map_err(|error| format!("Parsing image - {}", error))?;

    Ok(image)
}

fn append_cache_file(date: NaiveDate, image_url: &str, cache_file: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cache_file)
        .map_err(|error| format!("Opening cache file - {}", error))?;

    writeln!(file, "{} {}", date, image_url)
        .map_err(|error| format!("Writing to cache file - {}", error))?;

    Ok(())
}

async fn fetch_image_url_from_date(
    client: &Client,
    date: NaiveDate,
    proxy: Option<&str>,
) -> Result<String, String> {
    let url = proxy::webpage_proxied(date, proxy);

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

    let Some(char_index) = response_body.find("https://assets.amuniversal.com") else {
        return Err(format!("Cannot find image URL in webpage body ({url})"));
    };

    let Some(image_url) = response_body.get(char_index..char_index + 63) else {
        return Err(format!(
            "Slicing text of webpage body for image URL ({url})"
        ));
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
