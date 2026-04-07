pub mod burn_credits;
pub mod clip_count;
pub mod download;
pub mod list_videos;
pub mod update;

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

use crate::{
    api::client::{ApiClient, CreateNewVideoRequest},
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

        Commands::BurnIntroText(args) => {
            crate::burner::intro_text::process_intro_text(&args, &config)?;
        }
        Commands::BurnOutroText(args) => {
            crate::burner::outro_text::process_outro_text(&args, &config)?;
        }

        Commands::Video(video_args) => match video_args.command {
            VideoCommands::Create { title } => {
                let response = api_client
                    .create_video(&CreateNewVideoRequest { title })
                    .await
                    .context("failed to create a video")?;

                if response.success {
                    if let Some(id) = response.video_id {
                        let manage_url = api_client
                            .base_url
                            .join(&format!("videos/{id}"))
                            .context("failed to join video id to base url (manage url)")?;

                        let submit_url = api_client
                            .base_url
                            .join(&format!("submit/{id}"))
                            .context("failed to join video id to base url (submit url)")?;

                        println!(
                            "\n  {}\n\n  {:<8} {}\n  {:<8} {}\n  {:<8} {}\n",
                            "successfully created new video".green().bold(),
                            "id:".bold(),
                            id.cyan(),
                            "manage:".bold(),
                            manage_url.as_str().blue().underline(),
                            "submit:".bold(),
                            submit_url.as_str().blue().underline()
                        );
                    } else {
                        // the server said success, but didn't provide an id (realistically this should never happen)
                        eprintln!("uh oh: server reported success but returned no video id");
                    }
                } else {
                    println!("failed to create video");
                }
            }
            VideoCommands::ClipCount { video_id } => {
                clip_count::handle(&api_client, &video_id).await?;
            }
            VideoCommands::Download { video_id } => {
                download::download_command(video_id, &config, &api_client).await?;
            }
            VideoCommands::BurnCredits { video_id, crf } => {
                burn_credits::burn_credits_cmd(&config, video_id, crf)?;
            }
        },

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
