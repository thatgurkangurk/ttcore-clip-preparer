use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;

use crate::{
    burner,
    config::Config,
    download,
    fs::{clean_burned_dirs, clean_output_dir, ensure_out_dir_exists},
    update,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Download {
        #[arg(long)]
        video_id: i32,
    },
    BurnCredits {
        #[arg(long)]
        video_id: i32,

        #[arg(long)]
        crf: Option<i32>,
    },
    Clean,
    CleanBurned,
    Update,
}

impl Commands {
    pub async fn execute(self, config_path: Option<PathBuf>) -> Result<()> {
        match self {
            Commands::Update => {
                update::update()?;
                return Ok(());
            }

            command => {
                let config =
                    Config::load(config_path.as_deref()).context("failed to load configuration")?;

                ensure_out_dir_exists(&config)
                    .await
                    .context("failed to ensure output directory exists")?;

                match command {
                    Commands::Download { video_id } => {
                        let client = Client::new();

                        download::download_selected_files(video_id, &config, &client)
                            .await
                            .context("download command failed")?;
                    }

                    Commands::BurnCredits { video_id, crf } => {
                        let path = config.fs.out_dir.join(video_id.to_string());

                        burner::burn_multiline_text_batch(path, config.fs.font_file, crf)
                            .context("failed to burn credits text")?;
                    }

                    Commands::Clean => {
                        clean_output_dir(&config)
                            .await
                            .context("failed to clean output directory")?;
                    }

                    Commands::CleanBurned => {
                        clean_burned_dirs(&config.fs.out_dir)
                            .await
                            .context("failed to clean burned directories")?;
                    }

                    Commands::Update => unreachable!(),
                }
            }
        }

        Ok(())
    }
}
