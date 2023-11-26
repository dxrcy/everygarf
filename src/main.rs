mod args;

use args::Args;
use clap::Parser;
use everygarf::get_folder_path;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let result = run_everything(args).await;

    if let Err(err) = result {
        eprintln!("error: {:#?}", err);
        std::process::exit(1);
    }
}

async fn run_everything(args: Args) -> Result<(), String> {
    let folder = get_folder_path(args.folder)?;
    everygarf::create_empty_target_directory(&folder)?;
    everygarf::download_all_images(&folder).await?;
    Ok(())
}
