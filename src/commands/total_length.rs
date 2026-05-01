use std::{sync::Arc, time::Duration};
use tokio::process::Command;
use tokio::task::JoinSet;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::tempdir;
use tokio::sync::Semaphore;

use crate::{api::client::ApiClient, download::download_file_into_temp_dir};

pub async fn handle(api_client: &ApiClient, video_id: &str) -> Result<Duration> {
    let temp_dir = Arc::new(tempdir()?);
    
    let res = api_client
        .list_selected_clips_for_video(video_id, true)
        .await
        .context("failed to fetch clips for video")?;

    let total_clips = res.clips.len();
    if total_clips == 0 {
        return Ok(Duration::ZERO);
    }

    let semaphore = Arc::new(Semaphore::new(5));

    let pb = ProgressBar::new(total_clips as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut set = JoinSet::new();
    
    for clip in res.clips {
        let client = api_client.client.clone();
        let url = clip.url.clone();
        let temp_dir_clone = Arc::clone(&temp_dir);
        let sem = Arc::clone(&semaphore);
        let pb_task = pb.clone();

        let permit = sem.acquire_owned().await.context("semaphore closed")?;

        set.spawn(async move {
            let _permit = permit; 

            let file_path = download_file_into_temp_dir(&url, &temp_dir_clone, &client).await?;
            
            let output = Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-show_entries", "format=duration",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                ])
                .arg(&file_path)
                .output()
                .await?;

            if !output.status.success() {
                anyhow::bail!("ffprobe failed for {}", file_path.display());
            }

            let duration_str = String::from_utf8_lossy(&output.stdout);
            let duration: f64 = duration_str.trim().parse()?;
            
            pb_task.inc(1); 
            
            Ok::<f64, anyhow::Error>(duration)
        });
    }

    let mut total_secs = 0.0;
    while let Some(task_result) = set.join_next().await {
        let clip_duration = task_result.context("task panicked")??;
        total_secs += clip_duration;
    }

    pb.finish_with_message("processing complete");
    
    let total_duration = Duration::from_secs_f64(total_secs);
    println!("total (approximate) duration (excluding intro, credit text, and stuff like that): {}", format_duration(total_duration));

    Ok(total_duration)
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}