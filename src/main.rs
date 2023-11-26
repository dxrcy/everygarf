mod args;

use args::Args;
use clap::Parser;
use everygarf::get_folder_path;
use humantime::format_duration;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("EveryGarf - Comic Downloader");
    let start_time = Instant::now();

    match run_downloads(args).await {
        Ok(download_count) => {
            let elapsed_time = format_duration(start_time.elapsed());

            println!("Complete!");
            println!(" • Downloaded: {} files", download_count);
            println!(" • Elapsed: {}", elapsed_time);
        }

        Err(err) => {
            eprintln!("error: {:#?}", err);
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

    if args.remove_all {
        println!("Removing all images in {}", folder_string);
    } else {
        println!("Checking for missing images in {}", folder_string);
    }
    everygarf::create_dir(&folder, args.remove_all)?;

    let all_dates = everygarf::get_all_dates();
    let existing_dates = everygarf::get_existing_dates(&folder)?;
    let missing_dates: Vec<_> = all_dates
        .into_iter()
        .filter(|date| !existing_dates.contains(date))
        .collect();

    if missing_dates.is_empty() {
        println!("Everything is up to date.");
        return Ok(0);
    }

    if args.count {
        println!(
            "There are {} missing images to download",
            missing_dates.len()
        );
        return Ok(0);
    }

    println!(
        "Downloading {} images using (up to) {} concurrent jobs...",
        missing_dates.len(),
        job_count,
    );

    everygarf::download_all_images(&folder, &missing_dates, job_count, attempt_count, request_timeout).await?;

    Ok(missing_dates.len())
}
