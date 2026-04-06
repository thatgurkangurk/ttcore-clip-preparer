use super::API_BASE_URL;
use crate::{
    api::{clips::ClipsResponse, videos::VideoListResponse},
    config::Config,
};
use anyhow::{Context, Result};
use reqwest::{Method, RequestBuilder};
use url::Url;

pub struct ApiClient {
    pub client: reqwest::Client,
    api_key: String,
    base_url: Url,
}

impl ApiClient {
    pub fn new(config: &Config) -> Self {
        let base_url = config
            .api
            .base_url
            .clone()
            .unwrap_or_else(|| Url::parse(API_BASE_URL).expect("valid default url"));

        Self {
            client: reqwest::Client::new(),
            api_key: config.api.key.clone(),
            base_url: base_url,
        }
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.base_url.join(path).expect("invalid path");

        self.client
            .request(method, url)
            .header("x-api-key", &self.api_key)
    }

    pub async fn list_clips_for_video(&self, video_id: &str) -> Result<ClipsResponse> {
        let res = self
            .request(Method::GET, &format!("/api/videos/{video_id}/list"))
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

    pub async fn list_videos(&self) -> Result<VideoListResponse> {
        let response = self
            .request(Method::GET, "/api/videos/list")
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
