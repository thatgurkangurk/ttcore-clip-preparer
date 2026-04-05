use crate::api::client::ApiClient;
use crate::config::Config;
use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use futures_util::StreamExt;
use futures_util::stream;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

async fn download_clip(
    client: &reqwest::Client,
    clip: &crate::api::clips::Clip,
    video_id: &str,
    base_dir: &Path,
    multi: &MultiProgress,
) -> anyhow::Result<()> {
    let pb = multi.add(ProgressBar::new(0));
    pb.set_style(
        ProgressStyle::with_template(
            "{msg:30!} [{bar:40.green/white}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )?
        .progress_chars("=>-"),
    );
    pb.set_message(format!("Downloading {}", clip.title));

    let author_name = clip.overridden_profile_data.as_ref().map_or_else(
        || clip.creator.username.clone(),
        |profile| format!("profile__{}", profile.line1),
    );

    let author_snake = author_name.to_case(Case::Snake);
    let video_dir = base_dir.join(video_id).join(&author_snake).join("video");
    let info_path = base_dir.join(video_id).join(&author_snake).join("info.txt");

    tokio::fs::create_dir_all(&video_dir).await?;

    let file_content = clip.overridden_profile_data.as_ref().map_or_else(
        || format!("{}\n@{}", clip.creator.name, clip.creator.username),
        |profile| format!("{}\n{}", profile.line1, profile.line2),
    );

    let mut info_file = tokio::fs::File::create(&info_path).await?;
    info_file.write_all(file_content.as_bytes()).await?;

    let filename = clip
        .url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|name| !name.is_empty())
        .unwrap_or("video.mp4");

    let path = video_dir.join(filename);

    let response = client
        .get(clip.url.clone())
        .send()
        .await?
        .error_for_status()?;

    if let Some(len) = response.content_length() {
        pb.set_length(len);
    }

    let mut file = tokio::fs::File::create(&path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message(format!("Saved {}", clip.title));

    Ok(())
}

/// downloads selected files from ttcore.gurkz.me
///
/// selected in this case means the ones marked on the frontend as "selected"
pub async fn download_selected_files(
    video_id: &str,
    config: &Config,
    api_client: &ApiClient,
) -> Result<()> {
    let client = Arc::new(api_client.client.clone());
    let base_dir = Arc::new(config.fs.out_dir.clone());

    let selected_clips = api_client
        .list_selected_clips_for_video(video_id, true)
        .await
        .context("failed to fetch clips")?
        .clips;

    let total_files = selected_clips.len() as u64;
    let multi = Arc::new(MultiProgress::new());

    let overall_pb = multi.add(ProgressBar::new(total_files));
    overall_pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files ({eta})",
        )?
        .progress_chars("##-"),
    );

    let video_id_owned = video_id.to_string();

    stream::iter(selected_clips)
        .for_each_concurrent(3, |clip| {
            let client = Arc::clone(&client);
            let base_dir = Arc::clone(&base_dir);
            let multi = Arc::clone(&multi);
            let overall_pb = overall_pb.clone();
            let video_id = video_id_owned.clone();

            async move {
                match download_clip(&client, &clip, &video_id, &base_dir, &multi).await {
                    Ok(()) => overall_pb.inc(1),
                    Err(e) => eprintln!("Failed to download {}: {e:#}", clip.title),
                }
            }
        })
        .await;

    overall_pb.finish_with_message("All downloads complete");

    Ok(())
}
