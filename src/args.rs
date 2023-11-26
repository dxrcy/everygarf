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

    /// Don't send desktop notifications on error
    #[arg(short, long)]
    pub quiet: bool,

    /// Remove existing files / clean save folder (not recommended)
    #[arg(long)]
    pub remove_all: bool,

    /// Timeout for HTTP requests
    #[arg(short, long, default_value_t = 15)]
    pub timeout: u64,

    /// Amount of fetch attempts allowed per thread, before hard error
    #[arg(short, long, default_value_t = 10)]
    pub attempts: u32,

    /// Max concurrent jobs to run
    #[arg(short, long)]
    pub jobs: Option<usize>,
}
