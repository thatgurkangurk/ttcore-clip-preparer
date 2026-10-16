use std::borrow::Cow;
use std::time::Duration;

use crate::api::client::ApiClient;
use crate::burner::credits::{EncodeTask, run_ffmpeg};
use crate::commands::args::BurnSingleClipArgs;
use crate::config::Config;
use crate::download::{UserInfo, download_file_into_temp_dir};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::tempdir;

pub async fn burn_single_clip_cmd(
    config: &Config,
    args: &BurnSingleClipArgs,
    api_client: &ApiClient,
) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}]")?
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message("burning...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let temp_dir = tempdir()?;
    let res = api_client.get_single_clip(&args.clip_id).await?;

    let user_info = res.clip.overridden_profile_data.as_ref().map_or_else(
        || UserInfo {
            user_id: Cow::Owned(res.clip.creator.id.clone()),
            display_name: Cow::Owned(res.clip.creator.name.clone()),
            username: Cow::Owned(format!("@{}", res.clip.creator.username)),
        },
        |profile| UserInfo {
            user_id: Cow::Owned(format!("profile_{}", profile.id)),
            display_name: Cow::Owned(profile.line1.clone()),
            username: Cow::Owned(profile.line2.clone()),
        },
    );

    pb.set_message(format!("burning {}...", res.clip.title));

    let video_path =
        download_file_into_temp_dir(&res.clip.url, &temp_dir, &api_client.client).await?;

    let encode_task = EncodeTask {
        input: video_path,
        output: args.output.clone(),
        user_info,
    };

    run_ffmpeg(&encode_task, &config.fs.font_file, None)?;

    pb.finish_with_message("burned successfully");

    Ok(())
}
