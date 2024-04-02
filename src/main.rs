mod args;

use clap::Parser;
use human_bytes::human_bytes;
use humantime::format_duration;
use std::{
    process,
    time::{Duration, Instant},
};

use crate::args::Args;
use everygarf::{colors::*, dates, errors, fatal_error, get_folder_path, DownloadOptions};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if !args.query {
        print!("{BOLD}");
        println!(" ┌─────────────┐");
        println!(" │  EveryGarf  │");
        println!(" └─────────────┘{RESET} {ITALIC}Comic Downloader{RESET}");
    }

    let start_time = Instant::now();
    let notify_fail = args.notify_fail;

    let folder = get_folder_path(args.folder.as_deref())
        .unwrap_or_else(|error| fatal_error(errors::NO_DIR, error, notify_fail));
    let folder_string = folder.to_string_lossy();

    let start_date = args.start_from.unwrap_or(dates::first());
    let timeout = Duration::from_secs(args.timeout.into());
    let timeout_initial = Duration::from_secs(args.initial_timeout.into());
    let job_count: usize = args.jobs.into();
    let attempt_count: u32 = args.attempts.into();

    if !args.query {
        println!(
            "{} in {UNDERLINE}{}{RESET}",
            if args.remove_all {
                "Removing all images"
            } else {
                "Checking for missing images"
            },
            folder_string
        );
    }

    everygarf::create_target_dir(&folder, args.remove_all)
        .map_err(|error| {
            format!(
                "Failed to create or clear target directory `{}` - {:#?}",
                folder_string, error,
            )
        })
        .unwrap_or_else(|error| fatal_error(errors::CREATE_DIR, error, notify_fail));

    let (first_date, today_date) = (dates::first(), dates::today());
    if start_date < first_date {
        fatal_error(
            errors::BAD_START_DATE,
            format!(
                "Start date must not be before date of first comic ({})",
                first_date,
            ),
            notify_fail,
        );
    }
    if start_date > today_date {
        fatal_error(
            errors::BAD_START_DATE,
            format!(
                "Start date must not be after today's date (UTC - {})",
                today_date,
            ),
            notify_fail,
        );
    }

    let all_dates = dates::get_dates_between(start_date, dates::latest());
    let existing_dates = everygarf::get_existing_dates(&folder)
        .unwrap_or_else(|error| fatal_error(errors::READ_EXISTING_DATES, error, notify_fail));

    let mut missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    let total_download_count = missing_dates.len();
    if args.query {
        if total_download_count > 0 {
            process::exit(errors::QUERY_SOME as i32);
        } else {
            process::exit(errors::QUERY_NONE as i32);
        }
    }
    if let Some(max) = args.max {
        println!(
            "There are {BOLD}{}{RESET} total missing images to download",
            total_download_count,
        );
        if total_download_count > 0 {
            println!("{CYAN}Note: {DIM}Run without {BOLD}--max{RESET}{CYAN}{DIM} argument to download all images{RESET}");
        }
        missing_dates.truncate(max);
    }
    let real_download_count = missing_dates.len();

    let proxy = if args.no_proxy {
        None
    } else {
        Some(args.proxy)
    };
    let cache_url = if args.no_cache {
        None
    } else {
        Some(args.cache)
    };
    let cache_file = args.save_cache;
    let image_format = args.format.to_string();

    let download_options = DownloadOptions {
        attempt_count,
        proxy: proxy.as_deref(),
        cache_file: cache_file.as_deref(),
        image_format: image_format.as_str(),
    };

    if real_download_count > 0 {
        println!(
            "Downloading {BOLD}{}{RESET} images using (up to) {BOLD}{}{RESET} concurrent jobs...{RESET}",
            missing_dates.len(),
            job_count,
        );
        everygarf::download_all_images(
            &folder,
            &missing_dates,
            job_count,
            [timeout, timeout_initial],
            notify_fail,
            cache_url,
            download_options,
        )
        .await;
    }

    let elapsed_time = format_duration(Duration::from_secs(start_time.elapsed().as_secs()));
    let folder_size = fs_extra::dir::get_size(folder)
        .map(|size| human_bytes(size as f64))
        .unwrap_or_else(|_| "???".into());

    println!();
    if total_download_count == 0 {
        println!("{GREEN}{BOLD}Everything is up to date!{RESET}");
    } else if real_download_count == 0 {
        println!("{GREEN}{BOLD}Nothing downloaded!{RESET}");
    } else {
        println!("{GREEN}{BOLD}Complete!{RESET}");
    }
    println!(
        " {DIM}•{RESET} Downloaded: {BOLD}{}{RESET} images",
        real_download_count,
    );
    println!(" {DIM}•{RESET} Elapsed time: {BOLD}{}{RESET}", elapsed_time);
    println!(" {DIM}•{RESET} Total size: {BOLD}{}{RESET}", folder_size);
    println!();
}
