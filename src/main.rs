mod api;
mod burner;
mod cli;
mod config;
mod download;
mod fs;
mod update;

use clap::Parser;

use crate::config::Config;
use crate::{cli::Cli, fs::clean_output_dir};
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
        Some(cli::Commands::Clean) => clean_output_dir(&config)
            .await
            .context("failed to clean the output directory")?,
        Some(cli::Commands::CleanBurned) => {
            let path = config.fs.out_dir;

            clean_burned_dirs(&path)
                .await
                .context("Failed to clean burned directories")?;
        }
        Some(cli::Commands::Update) => crate::update::update()?,
        None => {}
    }

    Ok(())
}
