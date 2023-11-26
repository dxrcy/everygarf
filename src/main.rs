mod args;

use clap::Parser;
use everygarf::get_folder_path;
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

    match run_downloads(args).await {
        Ok(download_count) => {
            let elapsed_time = format_duration(Duration::from_secs(start_time.elapsed().as_secs()));

            println!("{GREEN}Complete!{RESET}");
            println!(
                " {DIM}•{RESET} Downloaded: {BOLD}{}{RESET} files",
                download_count
            );
            println!(" {DIM}•{RESET} Elapsed time: {BOLD}{}{RESET}", elapsed_time,);
        }

        Err(err) => {
            eprintln!("{RED}-- {BOLD}ERROR{RESET} {RED}--{RESET}\n{:#?}", err);
            std::process::exit(1);
        }
    }
}

async fn run_downloads(args: Args) -> Result<usize, String> {
    let folder = get_folder_path(args.folder)?;
    let folder_string = folder.to_string_lossy();

    let request_timeout = Duration::from_secs(args.timeout);
    let job_count = args.jobs;
    let attempt_count = args.attempts;

    println!(
        "{} in {UNDERLINE}{}{RESET}",
        if args.remove_all {
            "Removing all images"
        } else {
            "Checking for missing images"
        },
        folder_string
    );
    everygarf::create_target_dir(&folder, args.remove_all).map_err(|err| {
        format!(
            "Failed to create or clear target directory `{}` - {:#?}",
            folder_string, err,
        )
    })?;

    let all_dates = everygarf::get_all_dates();
    let existing_dates = everygarf::get_existing_dates(&folder)?;
    let missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    if missing_dates.is_empty() {
        println!("{GREEN}Everything is up to date!{RESET}");
        return Ok(0);
    }

    if args.count {
        println!(
            "There are {BOLD}{}{RESET} missing images to download",
            missing_dates.len()
        );
        println!("{YELLOW}Note: {DIM}Run without {BOLD}--count{RESET}{YELLOW}{DIM} argument to start download{RESET}");
        return Ok(0);
    }

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
    )
    .await?;

    Ok(missing_dates.len())
}
