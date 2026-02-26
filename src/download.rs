use crate::config::Config;
use convert_case::{Case, Casing};
use futures_util::StreamExt;
use futures_util::stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
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
    config: &Config,
) -> Result<ClipsResponse, reqwest::Error> {
    let response = client
        .get(format!(
            "https://ttcore.gurkz.me/api/videos/{video_id}/list"
        ))
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
    let client = Arc::new(reqwest::Client::new());
    let base_dir = Arc::new(config.fs.out_dir.clone());

    // Fetch clips
    let clips = fetch_clips(&client, video_id, config).await?;

    // Filter selected clips
    let selected_clips: Vec<_> = clips.clips.into_iter().filter(|c| c.selected).collect();

    stream::iter(selected_clips)
        .for_each_concurrent(3, |clip| {
            let client = Arc::clone(&client);
            let base_dir = Arc::clone(&base_dir);
            async move {
                if let Err(e) = async {
                    println!("Downloading: {}", clip.title);

                    // Convert author name to snake_case
                    let author_snake = clip.creator.username.to_case(Case::Snake);

                    // Build directories
                    let video_dir = base_dir
                        .join(video_id.to_string())
                        .join(&author_snake)
                        .join("video");
                    tokio::fs::create_dir_all(&video_dir).await?;

                    // Write info.txt
                    let info_path = base_dir
                        .join(video_id.to_string())
                        .join(&author_snake)
                        .join("info.txt");
                    let mut info_file = tokio::fs::File::create(&info_path).await?;
                    info_file
                        .write_all(
                            format!("{}\n@{}", clip.creator.name, clip.creator.username).as_bytes(),
                        )
                        .await?;

                    // Determine filename from URL
                    let filename = clip
                        .url
                        .path_segments()
                        .and_then(|mut segments| segments.next_back())
                        .filter(|name| !name.is_empty())
                        .unwrap_or("video.mp4");

                    let path = video_dir.join(filename);

                    // Download the video
                    let response = client
                        .get(clip.url.clone())
                        .send()
                        .await?
                        .error_for_status()?;
                    let mut file = tokio::fs::File::create(&path).await?;
                    let mut stream = response.bytes_stream();

                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        file.write_all(&chunk).await?;
                    }

                    println!("Saved to {}", path.display());
                    Ok::<(), Box<dyn std::error::Error>>(())
                }
                .await
                {
                    eprintln!("Failed to download {}: {e}", clip.title);
                }
            }
        })
        .await;

    Ok(())
}
