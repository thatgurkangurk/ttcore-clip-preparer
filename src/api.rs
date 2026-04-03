use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipsResponse {
    pub clips: Vec<Clip>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub created_by_id: String,
    pub video_id: String,
    pub url: Url,
    pub title: String,
    pub selected: bool,
    pub created_at: String,
    pub overridden_profile_data_id: Option<String>,
    pub creator: Creator,
    pub overridden_profile_data: Option<OverriddenProfileData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverriddenProfileData {
    pub id: String,
    pub line1: String,
    pub line2: String,
}

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

const API_BASE_URL: &str = "https://ttcore.gurkz.me";

pub async fn fetch_clips_for_video(
    client: &reqwest::Client,
    video_id: &str,
    config: &Config,
) -> Result<ClipsResponse> {
    let response = client
        .get(format!("{API_BASE_URL}/api/videos/{video_id}/list"))
        .header("x-api-key", &config.api.key)
        .send()
        .await
        .context("failed to send request")?
        .error_for_status()
        .context("request returned error status")?;

    let clips = response
        .json::<ClipsResponse>()
        .await
        .context("failed to deserialise ClipsResponse")?;

    Ok(clips)
}

pub async fn fetch_videos(client: &reqwest::Client, config: &Config) -> Result<VideoListResponse> {
    let response = client
        .get(format!("{API_BASE_URL}/api/videos/list"))
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
