use super::API_BASE_URL;
use crate::{
    api::{
        clips::{ClipsResponse, GetSingleClipResponse},
        videos::VideoListResponse,
    },
    config::Config,
};
use anyhow::{Context, Result};
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize)]
pub struct CreateNewVideoRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewVideoResponse {
    pub success: bool,

    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

pub struct ApiClient {
    pub client: reqwest::Client,
    api_key: String,
    pub base_url: Url,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self> {
        let base_url = match &config.api.base_url {
            Some(url) => url.clone(),
            None => {
                Url::parse(API_BASE_URL).context("failed to parse default API_BASE_URL (how?)")?
            }
        };

        Ok(Self {
            client: reqwest::Client::new(),
            api_key: config.api.key.clone(),
            base_url,
        })
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = self
            .base_url
            .join(path)
            .with_context(|| format!("failed to join URL with path: {path}"))?;

        Ok(self
            .client
            .request(method, url)
            .header("x-api-key", &self.api_key))
    }

    pub async fn list_clips_for_video(&self, video_id: &str) -> Result<ClipsResponse> {
        let res = self
            .request(Method::GET, &format!("/api/videos/{video_id}/list"))?
            .send()
            .await
            .context("failed to send request")?
            .error_for_status()
            .context("request returned error status")?
            .json::<ClipsResponse>()
            .await
            .context("failed to deserialise ClipsResponse")?;

        Ok(res)
    }

    pub async fn create_video(
        &self,
        payload: &CreateNewVideoRequest,
    ) -> Result<CreateNewVideoResponse> {
        let res = self
            .request(Method::POST, "/api/videos/create")?
            .json(payload)
            .send()
            .await
            .context("failed to send create video request")?
            .error_for_status()
            .context("create video request returned an error status")?
            .json::<CreateNewVideoResponse>()
            .await
            .context("failed to deserialise CreateNewVideoResponse")?;

        Ok(res)
    }

    pub async fn list_selected_clips_for_video(
        &self,
        video_id: &str,
        selected_only: bool,
    ) -> Result<ClipsResponse> {
        let res = self.list_clips_for_video(video_id).await?;

        let res = if selected_only {
            ClipsResponse {
                clips: res.clips.into_iter().filter(|c| c.selected).collect(),
            }
        } else {
            res
        };

        Ok(res)
    }

    pub async fn get_single_clip(&self, clip_id: &str) -> Result<GetSingleClipResponse> {
        let response = self
            .request(Method::GET, &format!("/api/clips/{clip_id}"))?
            .send()
            .await
            .context("failed to send request")?
            .error_for_status()
            .context("request returned error status")?;

        let clip = response
            .json::<GetSingleClipResponse>()
            .await
            .context("failed to deserialise GetSingleClipResponse")?;

        Ok(clip)
    }

    pub async fn list_videos(&self) -> Result<VideoListResponse> {
        let response = self
            .request(Method::GET, "/api/videos/list")?
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
}
