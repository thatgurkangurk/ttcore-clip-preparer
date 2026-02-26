use crate::config::Config;
use tokio::io::AsyncWriteExt;
use url::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipsResponse {
    pub clips: Vec<Clip>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub created_by_id: String,
    pub video_id: i32,
    pub url: Url,
    pub title: String,
    pub selected: bool,
    pub created_at: String,
    pub creator: Creator,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub id: String,
    pub name: String,
    pub username: String,
}

async fn fetch_clips(
    client: &reqwest::Client,
    video_id: i32,
    config: &Config
) -> Result<ClipsResponse, reqwest::Error> {
    let response = client
        .get(format!("https://ttcore.gurkz.me/api/videos/{video_id}/list"))
        .header("x-api-key", &config.api.key)
        .send()
        .await?
        .error_for_status()?;

    let clips = response.json::<ClipsResponse>().await?;

    Ok(clips)
}

/// downloads selected files from ttcore.gurkz.me
/// 
/// selected in this case means the ones marked on the frontend as "selected"
pub async fn download_selected_files(
    video_id: i32,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let output_dir = &config.fs.out_dir;

    tokio::fs::create_dir_all(output_dir).await?;

    let clips = fetch_clips(&client, video_id, config).await?;

    for clip in clips.clips.into_iter().filter(|c| c.selected) {
        println!("Downloading: {}", clip.title);

        let response = client
            .get(clip.url.clone())
            .send()
            .await?
            .error_for_status()?;

        let filename = clip
            .url
            .path_segments()
            .and_then(|segments| segments.last())
            .filter(|name| !name.is_empty())
            .unwrap_or("video.mp4");

        let path = output_dir.join(filename);

        let mut file = tokio::fs::File::create(&path).await?;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        println!("Saved to {}", path.display());
    }

    Ok(())
}