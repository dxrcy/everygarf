use clap::Parser;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

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

    /// Maximum number of images to download
    #[arg(short, long, default_value = None)]
    pub max: Option<usize>,

    #[arg(short, long, default_value = None)]
    pub start_from: Option<chrono::NaiveDate>,

    /// Send desktop notifications on error
    #[arg(short, long)]
    pub notify_fail: bool,

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

    /// Url of custom proxy service (see README)
    #[arg(long, conflicts_with = "no_proxy", default_value = everygarf::url::PROXY_DEFAULT)]
    pub proxy: String,

    /// Do not use a proxy service (see README)
    #[arg(long, conflicts_with = "proxy")]
    pub no_proxy: bool,
}
