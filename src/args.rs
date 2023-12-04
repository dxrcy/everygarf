use clap::Parser;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

/// EveryGarf Comic Downloader
///
/// Concurrently download every Garfield comic to date
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Folder to download images into
    ///
    /// Leave blank to use 'garfield' folder in user pictures directory (~/Pictures/garfield)
    pub folder: Option<String>,

    /// Maximum number of images to download
    ///
    /// Use `--max 0` to download no images, only check status
    #[arg(short, long, default_value = None)]
    pub max: Option<usize>,

    /// Only download comics published after this date (inclusive)
    ///
    /// Use `--max 1 --start_from <DATE>` to download 1 specific comic
    #[arg(short, long, default_value = None)]
    pub start_from: Option<chrono::NaiveDate>,

    /// Maximum number of concurrent jobs to run
    ///
    /// More jobs = faster, but is bottlenecked by network speed after a point
    #[arg(short, long, default_value_t = NonZeroUsize::new(20).unwrap())]
    pub jobs: NonZeroUsize,

    /// Timeout for HTTP requests (seconds)
    #[arg(short, long, default_value_t = NonZeroU64::new(10).unwrap())]
    pub timeout: NonZeroU64,

    /// Amount of fetch attempts allowed per thread, before hard error
    #[arg(short, long, default_value_t = NonZeroU32::new(10).unwrap())]
    pub attempts: NonZeroU32,

    /// Send desktop notifications on error
    ///
    /// Useful when running in background
    #[arg(short, long)]
    pub notify_fail: bool,

    /// Remove existing files / clean save folder (not recommended)
    ///
    /// Contributes to 'elapsed time'
    #[arg(long)]
    pub remove_all: bool,

    /// Url of custom proxy service
    ///
    /// See [https://github.com/darccyy/everygarf#proxy-service] for more information
    #[arg(long, conflicts_with = "no_proxy", default_value = everygarf::PROXY_DEFAULT)]
    pub proxy: String,

    /// Do not use a proxy service (not recommended)
    ///
    /// See [https://github.com/darccyy/everygarf#proxy-service] for more information
    #[arg(long, conflicts_with = "proxy")]
    pub no_proxy: bool,

    #[arg(long, short, conflicts_with = "no_cache", default_value = everygarf::CACHE_DEFAULT)]
    pub cache_url: String,

    #[arg(long, conflicts_with = "cache_url")]
    pub no_cache: bool,

    #[arg(long, conflicts_with = "no_cache")]
    pub save_cache: Option<String>,
}
