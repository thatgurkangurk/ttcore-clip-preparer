use anyhow::{Context, Result};
use reqwest::Client;

use crate::{api, config::Config};

pub async fn handle(client: &Client, config: &Config, video_id: &str) -> Result<()> {
    let res = api::clips::fetch_clips_for_video(client, video_id, config)
        .await
        .context("failed to fetch clips for video")?;

    let count = res.clips.len();
    let selected_count = res.clips.iter().filter(|p| p.selected).count();

    println!("The video has {count} clips, {selected_count} of which are selected.");

    Ok(())
}
