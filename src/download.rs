use bytes::Bytes;
use chrono::NaiveDate;
use image::DynamicImage;
use reqwest::Client;
use std::path::Path;

use crate::cache;
use crate::colors::*;
use crate::dates::date_to_string;
use crate::format_request_error;
use crate::proxy;
use crate::DateUrlCached;
use crate::DownloadOptions;
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
    download_options: DownloadOptions<'a>,
) -> Result<(), String> {
    let DownloadOptions {
        attempt_count,
        proxy,
        cache_file,
        image_format,
    } = download_options;

    let filename = date_to_string(date_cached.date, "-", true) + "." + image_format;
    let filename = Path::new(&filename);
    let filepath = folder.join(filename);

    for attempt_no in 1..=attempt_count {
        let result =
            fetch_image(client, &date_cached, job_id, total_count, proxy, cache_file).await;
        match result {
            Ok(image) => {
                if let Err(error) = image.save(filepath) {
                    return Err(format!(
                        "{} Failed to save image file - {error}",
                        date_cached.date,
                    ));
                }
                unsafe { PROGRESS_COUNT += 1 }
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
    total_count: usize,
    proxy: Option<&str>,
    cache_file: Option<&str>,
) -> Result<DynamicImage, String> {
    let image_url = match &date_cached.url {
        Some(url) => url.to_owned(),
        None => {
            print_step(date_cached.date, job_id, 1, total_count);
            fetch_image_url_from_date(client, date_cached.date, proxy)
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

    const IMAGE_URL_BASE: &str = "https://static.wikia.nocookie.net";

    let Some(char_index_left) = response_body.find(IMAGE_URL_BASE) else {
        return Err(format!("Cannot find image URL in webpage body ({url})"));
    };

    let mut char_index_right = char_index_left + IMAGE_URL_BASE.len() + 1;
    let mut chars = response_body.chars().skip(char_index_right);
    while chars.next().is_some_and(|ch| ch != '"') {
        char_index_right += 1;
    }

    let Some(image_url) = response_body.get(char_index_left..char_index_right) else {
        return Err(format!(
            "Slicing text of webpage body for image URL ({url})"
        ));
    };
    println!("{}", image_url);

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
