use crate::api;
use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub title: String,
    pub submissions_open: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoListResponse {
    pub videos: Vec<Video>,
}

pub async fn fetch_videos(client: &reqwest::Client, config: &Config) -> Result<VideoListResponse> {
    let response = client
        .get(format!("{}/api/videos/list", api::API_BASE_URL))
        .header("x-api-key", &config.api.key)
        .send()
        .await
        .context("failed to send request")?
        .error_for_status()
        .context("request returned error status")?;

    let videos = response
        .json::<VideoListResponse>()
        .await
        .context("failed to deserialise VideoListResponse")?;

    Ok(videos)
}
