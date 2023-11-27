mod args;

use clap::Parser;
use everygarf::{fatal_error, get_folder_path};
use human_bytes::human_bytes;
use humantime::format_duration;
use std::time::{Duration, Instant};

use crate::args::Args;
use everygarf::colors::*;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(" {BOLD}┌─────────────┐{RESET}");
    println!(" {BOLD}│  EveryGarf  │{RESET}");
    println!(" {BOLD}└─────────────┘{RESET} {ITALIC}Comic Downloader{RESET}");

    let start_time = Instant::now();
    let notify = args.notify_error;
    let folder = get_folder_path(args.folder.as_ref().map(String::as_str))
        .unwrap_or_else(|err| fatal_error(2, err, notify));

    let folder_string = folder.to_string_lossy();

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
        .map_err(|err| {
            format!(
                "Failed to create or clear target directory `{}` - {:#?}",
                folder_string, err,
            )
        })
        .unwrap_or_else(|err| fatal_error(2, err, notify));

    let all_dates = everygarf::get_all_dates();
    let existing_dates =
        everygarf::get_existing_dates(&folder).unwrap_or_else(|err| fatal_error(2, err, notify));
    let missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    let download_count = if missing_dates.is_empty() {
        println!("{GREEN}Everything is up to date!{RESET}");
        0
    } else if args.count {
        println!(
            "There are {BOLD}{}{RESET} missing images to download",
            missing_dates.len()
        );
        println!("{YELLOW}Note: {DIM}Run without {BOLD}--count{RESET}{YELLOW}{DIM} argument to start download{RESET}");
        0
    } else {
        println!(
            "Downloading {BOLD}{}{RESET} images using (up to) {BOLD}{}{RESET} concurrent jobs...{RESET}",
            missing_dates.len(),
            job_count,
        );
        println!("{DIM}Note: Downloads may not be in order{RESET}");
        everygarf::download_all_images(
            &folder,
            &missing_dates,
            job_count,
            attempt_count,
            request_timeout,
            notify,
        )
        .await;

        missing_dates.len()
    };

    let elapsed_time = format_duration(Duration::from_secs(start_time.elapsed().as_secs()));
    let folder_size = fs_extra::dir::get_size(folder)
        .map(|size| human_bytes(size as f64))
        .unwrap_or_else(|_| "???".into());

    println!("{GREEN}Complete!{RESET}");
    println!(
        " {DIM}•{RESET} Downloaded: {BOLD}{}{RESET} images",
        download_count
    );
    println!(" {DIM}•{RESET} Elapsed time: {BOLD}{}{RESET}", elapsed_time);
    println!(" {DIM}•{RESET} Total size: {BOLD}{}{RESET}", folder_size);
}
