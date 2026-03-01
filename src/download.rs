use crate::config::Config;
use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use futures_util::StreamExt;
use futures_util::stream;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

/// downloads selected files from ttcore.gurkz.me
///
/// selected in this case means the ones marked on the frontend as "selected"
pub async fn download_selected_files(
    video_id: i32,
    config: &Config,
    client: &reqwest::Client,
) -> Result<()> {
    let client = Arc::new(client);
    let base_dir = Arc::new(config.fs.out_dir.clone());

    let clips = crate::api::fetch_clips_for_video(&client, video_id, config)
        .await
        .context("failed to fetch clips")?;

    let selected_clips: Vec<_> = clips.clips.into_iter().filter(|c| c.selected).collect();
    let total_files = selected_clips.len() as u64;

    let multi = Arc::new(MultiProgress::new());

    // Global progress bar
    let overall_pb = multi.add(ProgressBar::new(total_files));
    overall_pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files ({eta})",
        )?
        .progress_chars("##-"),
    );

    stream::iter(selected_clips)
        .for_each_concurrent(3, |clip| {
            let client = Arc::clone(&client);
            let base_dir = Arc::clone(&base_dir);
            let multi = Arc::clone(&multi);
            let overall_pb = overall_pb.clone();

            async move {
                if let Err(e) = async {
                    // Create per-file progress bar
                    let pb = multi.add(ProgressBar::new(0));
                    pb.set_style(
                        ProgressStyle::with_template(
                            "{msg:30!} [{bar:40.green/white}] {bytes}/{total_bytes} \
                             ({bytes_per_sec}, {eta})",
                        )?
                        .progress_chars("=>-"),
                    );

                    pb.set_message(format!("Downloading {}", clip.title));

                    let author_snake = clip.creator.username.to_case(Case::Snake);

                    let video_dir = base_dir
                        .join(video_id.to_string())
                        .join(&author_snake)
                        .join("video");

                    tokio::fs::create_dir_all(&video_dir).await?;

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

                    // Set content length if known
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
                    overall_pb.inc(1);

                    Ok::<(), Box<dyn std::error::Error>>(())
                }
                .await
                {
                    eprintln!("Failed to download {}: {e}", clip.title);
                }
            }
        })
        .await;

    overall_pb.finish_with_message("All downloads complete");

    Ok(())
}
