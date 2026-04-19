use std::borrow::Cow;
use std::path::PathBuf;

use crate::api::client::ApiClient;
use crate::burner::credits::{EncodeTask, run_ffmpeg};
use crate::config::Config;
use crate::download::UserInfo;
use anyhow::Result;
use clap::Args;
use futures_util::StreamExt;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;

#[derive(Args, Debug, Clone)]
pub struct BurnSingleClipArgs {
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub clip_id: String,
}

pub async fn burn_single_clip_cmd(
    config: &Config,
    args: &BurnSingleClipArgs,
    api_client: &ApiClient,
) -> Result<()> {
    println!("burning");
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

    let video_path = temp_dir.path().join(format!("{}.mp4", args.clip_id));

    let response = api_client
        .client
        .get(res.clip.url)
        .send()
        .await?
        .error_for_status()?;

    let mut file = tokio::fs::File::create(&video_path).await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    let encode_task = EncodeTask {
        input: video_path,
        output: args.output.clone(),
        user_info,
    };

    run_ffmpeg(&encode_task, &config.fs.font_file, None)?;

    println!("done");

    Ok(())
}
