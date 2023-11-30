use bytes::Bytes;
use chrono::NaiveDate;
use image::DynamicImage;
use reqwest::Client;
use std::path::Path;

use crate::colors::*;
use crate::dates::date_to_string;
use crate::format_request_error;
use crate::url;

fn print_step(date: NaiveDate, job_id: usize, step: u32) {
    let icon = if step == 3 { "✓" } else { " " };
    let step = format!(
        "{}{step}{DIM}{}{RESET}",
        " ".repeat(step.max(1) as usize - 1),
        "•".repeat(3 - step.min(3) as usize),
    );
    println!(
        "    {BOLD}{date}{RESET}  {DIM}#{job_id:02}{RESET}  {BLUE}[{step}{BLUE}]{RESET} {GREEN}{icon}{RESET}"
    );
}

pub async fn download_image(
    client: &Client,
    date: NaiveDate,
    folder: &Path,
    job_id: usize,
    attempt_count: u32,
    use_proxy: bool,
) -> Result<(), String> {
    let filename = date_to_string(date, "-", true) + ".png";
    let filename = Path::new(&filename);
    let filepath = folder.join(filename);

    for attempt_no in 1..=attempt_count {
        let result = fetch_image(client, date, job_id, use_proxy).await;
        match result {
            Ok(image) => {
                if let Err(error) = image.save(filepath) {
                    return Err(format!("{date} Failed to save image file - {error}"));
                }
                break;
            }
            Err(error) => {
                eprintln!("{YELLOW}[warning] {DIM}[Attempt {attempt_no}]{RESET} {BOLD}{date}{RESET} {DIM}#{job_id}{RESET} Failed: {error}");
                if attempt_no >= attempt_count {
                    return Err(format!(
                        "{RESET}{BOLD}{date}{RESET} Failed after {BOLD}{attempt_count}{RESET} attempts: {error}"
                    ));
                }
            }
        }
    }

    Ok(())
}

async fn fetch_image(
    client: &Client,
    date: NaiveDate,
    job_id: usize,
    use_proxy: bool,
) -> Result<DynamicImage, String> {
    print_step(date, job_id, 1);
    let image_url = fetch_image_url_from_date(client, date, use_proxy)
        .await
        .map_err(|error| format!("Fetching image url - {}", error))?;

    print_step(date, job_id, 2);
    let image_bytes = fetch_image_bytes_from_url(client, &image_url)
        .await
        .map_err(|error| format!("Fetching image bytes - {}", error))?;

    print_step(date, job_id, 3);
    let image = image::load_from_memory(&image_bytes)
        .map_err(|error| format!("Parsing image - {}", error))?;

    Ok(image)
}

async fn fetch_image_url_from_date(
    client: &Client,
    date: NaiveDate,
    use_proxy: bool,
) -> Result<String, String> {
    let url = if use_proxy {
        url::webpage_proxied(date)
    } else {
        url::webpage_unproxied(date)
    };

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(format_request_error)?
        .error_for_status()
        .map_err(format_request_error)?;

    let response_body = response.text().await.map_err(|error| {
        format!("Converting webpage body for image url to text ({url}) - {error}")
    })?;

    let Some(char_index) = response_body.find("https://assets.amuniversal.com") else {
        return Err(format!("Cannot find image url in webpage body ({url})"));
    };

    let Some(image_url) = response_body.get(char_index..char_index + 63) else {
        return Err(format!(
            "Slicing text of webpage body for image url ({url})"
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
