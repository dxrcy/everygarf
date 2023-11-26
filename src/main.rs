use std::path::Path;

#[tokio::main]
async fn main() {
    let result = run_everything().await;

    if let Err(err) = result {
        eprintln!("error: {:#?}", err);
        std::process::exit(1);
    }
}

async fn run_everything() -> Result<(), String> {
    let folder = Path::new("./garfield");
    everygarf::create_empty_target_directory(folder)?;
    everygarf::download_all_images(&folder).await?;
    Ok(())
}
