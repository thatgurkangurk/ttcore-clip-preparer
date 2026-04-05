use anyhow::{Context, Result};

use crate::api::client::ApiClient;

pub async fn handle(api_client: &ApiClient, video_id: &str) -> Result<()> {
    let res = api_client
        .list_selected_clips_for_video(video_id, false)
        .await
        .context("failed to fetch clips for video")?;

    let count = res.clips.len();
    let selected_count = res.clips.iter().filter(|p| p.selected).count();

    println!("The video has {count} clips, {selected_count} of which are selected.");

    Ok(())
}
