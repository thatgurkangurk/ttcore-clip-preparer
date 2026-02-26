mod burner;
mod cli;
mod config;
mod download;

use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;
use std::path::Path;
use tokio::fs;
use tokio::io;

async fn ensure_out_dir_exists(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = &config.fs.out_dir;

    tokio::fs::create_dir_all(&path).await?;

    Ok(())
}

async fn clean_burned_dirs(base_folder: &Path) -> io::Result<()> {
    let mut level1 = fs::read_dir(base_folder).await?;

    while let Some(entry1) = level1.next_entry().await? {
        let path1 = entry1.path();
        if !path1.is_dir() {
            continue;
        }

        let mut level2 = fs::read_dir(&path1).await?;

        while let Some(entry2) = level2.next_entry().await? {
            let path2 = entry2.path();
            if !path2.is_dir() {
                continue;
            }

            let mut level3 = fs::read_dir(&path2).await?;

            while let Some(entry3) = level3.next_entry().await? {
                let path3 = entry3.path();

                if path3.is_dir()
                    && path3
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| name == "burned")
                {
                    fs::remove_dir_all(&path3).await?;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load().expect("expected config to load");

    ensure_out_dir_exists(&config).await?;

    match cli.command {
        Some(cli::Commands::Download { video_id }) => {
            download::download_selected_files(video_id, &config).await?;
        }
        Some(cli::Commands::BurnCredits { video_id }) => {
            let path = config.fs.out_dir.join(video_id.to_string());
            crate::burner::burn_multiline_text_batch(path, config.fs.font_file)?;
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
        Some(cli::Commands::CleanBurned) => {
            let path = config.fs.out_dir;

            clean_burned_dirs(&path).await?;
        }
        None => {}
    }

    Ok(())
}
