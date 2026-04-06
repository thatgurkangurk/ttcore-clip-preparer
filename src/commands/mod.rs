pub mod burn_credits;
pub mod clip_count;
pub mod download;
pub mod list_videos;
pub mod update;

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::{
    api::client::ApiClient,
    cli::{Commands, VideoCommands},
    config::Config,
    fs::{clean_burned_dirs, clean_output_dir, ensure_out_dir_exists},
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

    let api_client = ApiClient::new(&config)?;

    match command {
        Commands::ListVideos => list_videos::handle(&api_client).await?,

        Commands::Video(video_args) => {
            let video_id = video_args.video_id;

            match video_args.command {
                VideoCommands::ClipCount => {
                    clip_count::handle(&api_client, &video_id).await?;
                }
                VideoCommands::Download => {
                    download::download_command(video_id, &config, &api_client).await?;
                }
                VideoCommands::BurnCredits { crf } => {
                    burn_credits::burn_credits_cmd(&config, video_id, crf)?;
                }
            }
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
