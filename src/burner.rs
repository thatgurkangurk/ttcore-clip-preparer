use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn burn_multiline_text_batch(base_folder: PathBuf, font_file: PathBuf) -> Result<()> {
    let mut tasks: Vec<(PathBuf, PathBuf, String)> = Vec::new();

    // Collect all video jobs first
    for user_entry in fs::read_dir(&base_folder)? {
        let user_entry = user_entry?;
        let user_path = user_entry.path();

        if !user_path.is_dir() {
            continue;
        }

        let info_path = user_path.join("info.txt");
        let video_folder = user_path.join("video");

        if !info_path.exists() || !video_folder.exists() {
            continue;
        }

        let text = match fs::read_to_string(&info_path) {
            Ok(t) => t,
            Err(_) => continue,
        };

        for video_entry in fs::read_dir(&video_folder)? {
            let video_entry = video_entry?;
            let video_path = video_entry.path();

            if video_path.is_file() {
                let filename = match video_path.file_name() {
                    Some(f) => f.to_owned(),
                    None => continue,
                };

                let burned_dir = user_path.join("burned");
                fs::create_dir_all(&burned_dir)?;

                let output_video = burned_dir.join(filename);

                tasks.push((video_path, output_video, text.clone()));
            }
        }
    }

    let pb = ProgressBar::new(tasks.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} ({eta}) {msg}",
        )?
        .progress_chars("##-"),
    );

    let time_regex = Regex::new(r"time=(\d+:\d+:\d+\.\d+)")?;

    for (video_path, output_video, text) in tasks {
        let escaped_text = text
            .replace("\\", "\\\\")
            .replace(":", "\\:")
            .replace("'", "\\'")
            .trim()
            .to_string();

        let font_path = font_file
            .to_string_lossy()
            .to_string()
            .replace("\\", "\\\\");

        let drawtext_filter = format!(
            "drawtext=\
fontfile='{}':\
text='{}':\
fontcolor=white@0.75:\
fontsize=24:\
x=w-tw-50:\
y=h-th-40:\
line_spacing=6:\
text_align=2",
            font_path, escaped_text
        );

        pb.set_message(format!(
            "Encoding {}",
            video_path.file_name().unwrap().to_string_lossy()
        ));

        let mut cmd = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &video_path.to_string_lossy(),
                "-vf",
                &drawtext_filter,
                "-c:a",
                "copy",
                "-progress",
                "pipe:2",
                "-nostats",
                &output_video.to_string_lossy(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = cmd.stderr.take().context("Failed to capture stderr")?;
        let reader = BufReader::new(stderr);

        for line in reader.lines() {
            let line = line?;

            if let Some(cap) = time_regex.captures(&line) {
                pb.set_message(format!("time {}", &cap[1]));
            }

            if line.contains("progress=end") {
                break;
            }
        }

        let status = cmd.wait()?;

        if !status.success() {
            eprintln!("FFmpeg failed for {:?}", video_path);
        }

        pb.inc(1);
    }

    pb.finish_with_message("All videos processed 🎬");

    Ok(())
}
