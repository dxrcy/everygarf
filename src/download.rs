use bytes::Bytes;
use chrono::NaiveDate;
use image::DynamicImage;
use reqwest::{Client, StatusCode};
use std::path::Path;

use crate::colors::*;
use crate::dates::date_to_string;

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
) -> Result<(), String> {
    let filename = date_to_string(date, "-", true) + ".png";
    let filename = Path::new(&filename);
    let filepath = folder.join(filename);

    for attempt_no in 1..=attempt_count {
        let result = fetch_image(client, date, job_id).await;
        match result {
            Ok(image) => {
                if let Err(err) = image.save(filepath) {
                    return Err(format!("{date} Failed to save image file - {err}"));
                }
                break;
            }
            Err(err) => {
                eprintln!("{YELLOW}[warning] {DIM}[Attempt {attempt_no}]{RESET} {BOLD}{date}{RESET} {DIM}#{job_id}{RESET} Failed: {err}");
                if attempt_no >= attempt_count {
                    return Err(format!(
                        "{BOLD}{date}{RESET} Failed after {attempt_count} attempts: {err}"
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
) -> Result<DynamicImage, String> {
    print_step(date, job_id, 1);
    let image_url = fetch_image_url_from_date(client, date)
        .await
        .map_err(|err| format!("Fetching image url - {:#?}", err))?;

    print_step(date, job_id, 2);
    let image_bytes = fetch_image_bytes_from_url(client, &image_url)
        .await
        .map_err(|err| format!("Fetching image bytes - {:#?}", err))?;

    print_step(date, job_id, 3);
    let image = image::load_from_memory(&image_bytes)
        .map_err(|err| format!("Parsing image - {:#?}", err))?;

    Ok(image)
}

async fn fetch_image_url_from_date(client: &Client, date: NaiveDate) -> Result<String, String> {
    let date_string = date_to_string(date, "/", false);

    let url = format!(
        "https://corsproxy.garfieldapp.workers.dev/cors-proxy?https://www.gocomics.com/garfield/{}",
        date_string
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| format!("Fetching webpage body for image url ({url}) - {err}"))?
        .error_for_status()
        .map_err(format_status_error)?;

    let response_body = response
        .text()
        .await
        .map_err(|err| format!("Converting webpage body for image url to text ({url}) - {err}"))?;

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
        .map_err(|err| format!("Fetching image from url ({url}) - {err}"))?
        .error_for_status()
        .map_err(format_status_error)?;

    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("Converting image response to bytes ({url}) - {err}"))?;

    Ok(bytes)
}

fn format_status_error(error: reqwest::Error) -> String {
    let Some(status) = error.status() else {
        return format!("Unknown error: {:#?}", error);
    };
    let code = status.as_u16();

    let message = match (status, code) {
        (StatusCode::TOO_MANY_REQUESTS, _) => {
            format!("{CYAN}Rate limited.{RESET} Try again in a few minutes, or change IP.")
        }
        (_, 525) => format!("SSL handshake failed with Cloudflare."),
        _ => return format!("{YELLOW}{}{RESET}", error),
    };

    format!("{YELLOW}{}{RESET} {}", code, message)
}
