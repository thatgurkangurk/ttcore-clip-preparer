mod api;
mod burner;
mod cli;
mod config;
mod download;
mod update;
mod fs;

use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;
use anyhow::{Context, Result};

use crate::fs::{clean_burned_dirs, ensure_out_dir_exists};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load().context("failed to load configuration")?;
    let reqwest_client = reqwest::Client::new();

    ensure_out_dir_exists(&config)
        .await
        .context("failed to ensure that the output directory exists")?;

    match cli.command {
        Some(cli::Commands::Download { video_id }) => {
            download::download_selected_files(video_id, &config, &reqwest_client)
                .await
                .context("download command failed")?;
        }
        Some(cli::Commands::BurnCredits { video_id }) => {
            let path = config.fs.out_dir.join(video_id.to_string());
            crate::burner::burn_multiline_text_batch(path, config.fs.font_file)
                .context("failed to burn credits text")?;
        }
        Some(cli::Commands::Clean) => {
            let path = config.fs.out_dir;

            let mut entries = tokio::fs::read_dir(&path)
                .await
                .context("failed to read output directory")?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .context("failed to read directory entry")?
            {
                let path = entry.path();

                if path.is_dir() {
                    tokio::fs::remove_dir_all(&path)
                        .await
                        .context("failed to remove directory")?;
                } else {
                    tokio::fs::remove_file(&path)
                        .await
                        .context("failed to remove file")?;
                }
            }
        }
        Some(cli::Commands::CleanBurned) => {
            let path = config.fs.out_dir;

            clean_burned_dirs(&path)
                .await
                .context("Failed to clean burned directories")?;
        }
        Some(cli::Commands::Update) => {
            crate::update::update()?
        }
        None => {}
    }

    Ok(())
}
