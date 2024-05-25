use clap::{Parser, ValueEnum};
use std::{
    fmt::Display,
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
};

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

    /// Timeout for url and image requests (seconds)
    #[arg(short, long, default_value_t = NonZeroU64::new(5).unwrap())]
    pub timeout: NonZeroU64,

    /// Timeout for proxy ping and cache fetch (seconds)
    #[arg(short = 'T', long, default_value_t = NonZeroU64::new(10).unwrap())]
    pub initial_timeout: NonZeroU64,

    /// Amount of fetch attempts allowed per thread, before hard error
    #[arg(short, long, default_value_t = NonZeroU32::new(10).unwrap())]
    pub attempts: NonZeroU32,

    /// Send desktop notifications on error
    ///
    /// Useful when running in background
    #[arg(short, long)]
    pub notify_on_fail: bool,

    /// Remove existing files / clean save folder (not recommended)
    ///
    /// Contributes to 'elapsed time'
    #[arg(long)]
    pub remove_all: bool,

    /// Url of custom proxy service
    ///
    /// See [https://github.com/dxrcy/everygarf#proxy-service] for more information
    #[arg(long, conflicts_with = "no_proxy", default_value = everygarf::PROXY_DEFAULT)]
    pub proxy: String,

    /// Do not use a proxy service (not recommended)
    ///
    /// See [https://github.com/dxrcy/everygarf#proxy-service] for more information
    #[arg(long, conflicts_with = "proxy")]
    pub no_proxy: bool,

    /// Always ping proxy service, even when downloading few images
    #[arg(long)]
    pub always_ping: bool,

    /// Source for image URLs
    #[arg(long, requires = "no_cache", default_value_t = everygarf::api::Source::default())]
    pub source: everygarf::api::Source,

    /// Specify cache file to read from
    ///
    /// Disable cache with `no-cache`
    #[arg(short, long, default_value = everygarf::CACHE_DEFAULT, conflicts_with = "source")]
    pub cache: String,

    /// Do not read remote or local cache file
    #[arg(long, conflicts_with = "cache")]
    pub no_cache: bool,

    /// Save image URLs to local cache file
    ///
    /// Use cache file with `--cache <FILE>`
    ///
    /// Without `--no-cache`, this reuses the existing cached URLs
    #[arg(long)]
    pub save_cache: Option<String>,

    /// Image format (file extension) to save images as
    ///
    /// Format is ignored when files are checked for missing images, so no two images will have the
    /// same date, even if they have different file extensions
    #[arg(short, long, ignore_case = true, default_value_t = Default::default())]
    pub format: ImageFormat,

    /// Returns exit code 10 if images are missing
    ///
    /// Does not print anything to stdout
    #[arg(short, long)]
    pub query: bool,
}

/// File extension to save images as
#[derive(Default, Clone, Copy, ValueEnum)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpg,
    Gif,
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}
