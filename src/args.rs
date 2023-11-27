use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

use clap::Parser;

/// EveryGarf Comic Downloader
///
/// Download every Garfield comic, to date
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Folder to download images into
    ///
    /// Leave blank to use 'garfield' folder in user pictures directory (~/Pictures/garfield)
    pub folder: Option<String>,

    /// Calculate images that are to be downloaded, but don't download anything
    #[arg(short, long)]
    pub count: bool,

    /// Send desktop notifications on error
    #[arg(short, long)]
    pub notify_error: bool,

    /// Remove existing files / clean save folder (not recommended)
    #[arg(long)]
    pub remove_all: bool,

    /// Timeout for HTTP requests (seconds)
    #[arg(short, long, default_value_t = NonZeroU64::new(10).unwrap())]
    pub timeout: NonZeroU64,

    /// Maximum number of concurrent jobs to run
    #[arg(short, long, default_value_t = NonZeroUsize::new(20).unwrap())]
    pub jobs: NonZeroUsize,

    /// Amount of fetch attempts allowed per thread, before hard error
    #[arg(short, long, default_value_t = NonZeroU32::new(10).unwrap())]
    pub attempts: NonZeroU32,
}
