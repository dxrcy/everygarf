mod args;

use clap::Parser;
use std::{
    process,
    time::{Duration, Instant},
};

use crate::args::Args;
use everygarf::{
    api::Api, colors::*, dates, fatal_error, format_bytes, format_duration, get_dir_size,
    get_folder_path, Downloader, SingleDownloadOptions, Error,
};

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
    let notify_on_fail = args.notify_on_fail;

    let folder = get_folder_path(args.folder.as_deref())
        .unwrap_or_else(|error| fatal_error(Error::NoDir, error, notify_on_fail));
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
        .unwrap_or_else(|error| fatal_error(Error::CreateDir, error, notify_on_fail));

    let (first_date, today_date) = (dates::first(), dates::today());
    if start_date < first_date {
        fatal_error(
            Error::BadStartDate,
            format!(
                "Start date must not be before date of first comic ({})",
                first_date,
            ),
            notify_on_fail,
        );
    }
    if start_date > today_date {
        fatal_error(
            Error::BadStartDate,
            format!(
                "Start date must not be after today's date (UTC - {})",
                today_date,
            ),
            notify_on_fail,
        );
    }

    let all_dates = dates::get_dates_between(start_date, dates::latest());
    let existing_dates = everygarf::get_existing_dates(&folder)
        .unwrap_or_else(|error| fatal_error(Error::ReadExistingDates, error, notify_on_fail));

    let mut missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    let total_download_count = missing_dates.len();
    if args.query {
        let code = if total_download_count > 0 {
            everygarf::QUERY_SOME_EXITCODE
        } else {
            0
        };
        process::exit(code);
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

    let api = Api {
        source: args.source,
        proxy: proxy.as_deref(),
    };

    let cache_file = args.save_cache;
    let image_format = args.format.to_string();
    let always_ping = args.always_ping;

    let single_download_options = SingleDownloadOptions {
        attempt_count,
        api,
        cache_file: cache_file.as_deref(),
        image_format: image_format.as_str(),
        save_as_tree: args.tree,
    };

    let downloader = Downloader {
        single_download_options,
        folder: &folder,
        dates: &missing_dates,
        job_count,
        cache_url,
        always_ping,
        timeout_main: timeout,
        timeout_initial,
        notify_on_fail,
    };

    if real_download_count > 0 {
        println!(
            "Downloading {BOLD}{}{RESET} images using (up to) {BOLD}{}{RESET} concurrent jobs...{RESET}",
            missing_dates.len(),
            job_count,
        );
        downloader.download_all_images().await;
    }

    let elapsed_time = format_duration(Duration::from_secs(start_time.elapsed().as_secs()));
    let folder_size = get_dir_size(&folder)
        .map(format_bytes)
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
