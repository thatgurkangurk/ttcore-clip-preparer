use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use reqwest::Client;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Columns}
};

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
        video_id: String,
    },
    BurnCredits {
        #[arg(long)]
        video_id: String,

        #[arg(long)]
        crf: Option<i32>,
    },
    Clean,
    CleanBurned,
    Update,
    ListVideos,
}

#[derive(Tabled)]
struct VideoRow {
    id: String,
    title: String,
    submissions: String,
}

impl Commands {
    pub async fn execute(self, config_path: Option<PathBuf>) -> Result<()> {
        if matches!(self, Self::Update) {
            update::update()?;
            return Ok(());
        }

        // load config and client for all other commands
        let config =
            Config::load(config_path.as_deref()).context("failed to load configuration")?;
        ensure_out_dir_exists(&config)
            .await
            .context("failed to ensure output directory exists")?;
        let client = Client::new();

        match self {
            Self::ListVideos => {
                let res = crate::api::fetch_videos(&client, &config).await?;

                let rows: Vec<VideoRow> = res
        .videos
        .into_iter()
        .map(|v| {
            let submissions = if v.submissions_open {
                "OPEN".green().bold().to_string()
            } else {
                "CLOSED".red().bold().to_string()
            };

            VideoRow {
                id: v.id,
                title: v.title,
                submissions,
            }
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::modern())
        .with(Modify::new(Columns::one(0)).with(Alignment::center()))
        .with(Modify::new(Columns::one(1)).with(Alignment::left()))
        .with(Modify::new(Columns::one(2)).with(Alignment::center())).clone();

    println!("{table}");
            }
            Self::Download { video_id } => {
                download::download_selected_files(&video_id, &config, &client)
                    .await
                    .context("download command failed")?;
            }
            Self::BurnCredits { video_id, crf } => {
                burner::burn_multiline_text_batch(
                    &config.fs.out_dir.join(video_id),
                    &config.fs.font_file,
                    crf,
                )
                .context("failed to burn credits text")?;
            }
            Self::Clean => {
                clean_output_dir(&config)
                    .await
                    .context("failed to clean output directory")?;
            }
            Self::CleanBurned => {
                clean_burned_dirs(&config.fs.out_dir)
                    .await
                    .context("failed to clean burned directories")?;
            }
            Self::Update => unreachable!(), // already handled
        }

        Ok(())
    }
}
