mod cli;
mod config;
mod download;

use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;

async fn ensure_out_dir_exists(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = &config.fs.out_dir;

    tokio::fs::create_dir_all(&path).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::try_parse().expect("expected cli to parse");
    let config = Config::load().expect("expected config to load");

    ensure_out_dir_exists(&config).await?;

    match cli.command {
        Some(cli::Commands::Download { video_id }) => {
            download::download_selected_files(video_id, &config).await?;
        }
        Some(cli::Commands::Clean) => {
            let path = config.fs.out_dir;

            let mut entries = tokio::fs::read_dir(&path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    tokio::fs::remove_dir_all(&path).await?;
                } else {
                    tokio::fs::remove_file(&path).await?;
                }
            }
        }
        None => {}
    }

    Ok(())
}
