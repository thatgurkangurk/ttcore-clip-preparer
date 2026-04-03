pub mod clip_count;
pub mod list_videos;

use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;

use crate::{
    burner,
    cli::Commands,
    config::Config,
    download,
    fs::{clean_burned_dirs, clean_output_dir, ensure_out_dir_exists},
    update,
};

pub async fn execute(command: Commands, config_path: Option<PathBuf>) -> Result<()> {
    if matches!(command, Commands::Update) {
        update::update()?;
        return Ok(());
    }

    // load shared state for all other commands
    let config = Config::load(config_path.as_deref()).context("failed to load configuration")?;
    ensure_out_dir_exists(&config)
        .await
        .context("failed to ensure output directory exists")?;

    let client = Client::new();

    match command {
        Commands::ListVideos => list_videos::handle(&client, &config).await?,
        Commands::ClipCount { video_id } => clip_count::handle(&client, &config, &video_id).await?,
        Commands::Download { video_id } => {
            download::download_selected_files(&video_id, &config, &client)
                .await
                .context("download command failed")?;
        }
        Commands::BurnCredits { video_id, crf } => {
            burner::burn_multiline_text_batch(
                &config.fs.out_dir.join(video_id),
                &config.fs.font_file,
                crf,
            )
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

    Ok(())
}
