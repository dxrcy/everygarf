mod args;

use clap::Parser;
use everygarf::{fatal_error, get_folder_path};
use human_bytes::human_bytes;
use humantime::format_duration;
use std::time::{Duration, Instant};

use crate::args::Args;
use everygarf::{colors::*, dates};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(" {BOLD}┌─────────────┐{RESET}");
    println!(" {BOLD}│  EveryGarf  │{RESET}");
    println!(" {BOLD}└─────────────┘{RESET} {ITALIC}Comic Downloader{RESET}");

    let start_time = Instant::now();
    let notify_fail = args.notify_fail;

    let folder = get_folder_path(args.folder.as_deref())
        .unwrap_or_else(|error| fatal_error(2, error, notify_fail));
    let folder_string = folder.to_string_lossy();

    let start_date = args.start_from.unwrap_or(dates::first());
    let request_timeout = Duration::from_secs(args.timeout.into());
    let job_count: usize = args.jobs.into();
    let attempt_count: u32 = args.attempts.into();

    println!(
        "{} in {UNDERLINE}{}{RESET}",
        if args.remove_all {
            "Removing all images"
        } else {
            "Checking for missing images"
        },
        folder_string
    );
    everygarf::create_target_dir(&folder, args.remove_all)
        .map_err(|error| {
            format!(
                "Failed to create or clear target directory `{}` - {:#?}",
                folder_string, error,
            )
        })
        .unwrap_or_else(|error| fatal_error(3, error, notify_fail));

    let all_dates = dates::get_dates_between(start_date, dates::latest());
    let existing_dates = everygarf::get_existing_dates(&folder)
        .unwrap_or_else(|error| fatal_error(4, error, notify_fail));
    let mut missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    let total_download_count = missing_dates.len();
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
            attempt_count,
            request_timeout,
            notify_fail,
            !args.no_proxy,
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
