use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use std::io::Read;

use crate::download::UserInfo;

fn is_video_valid(path: &PathBuf) -> bool {
    // File must be reasonably sized (avoid partial writes)
    const MIN_SIZE_BYTES: u64 = 1024 * 100; // 100KB
    if !path.exists() {
        return false;
    }

    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() < MIN_SIZE_BYTES {
            return false;
        }
    } else {
        return false;
    }

    // Use ffprobe to verify container + streams
    let status = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "csv=p=0",
            path.to_string_lossy().as_ref(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

#[allow(clippy::too_many_lines)]
pub fn burn_multiline_text_batch(
    base_folder: &PathBuf,
    font_file: &Path,
    crf: Option<i32>,
) -> Result<()> {
    let mut tasks: Vec<(PathBuf, PathBuf, String, String)> = Vec::new();

    // Collect all video jobs first
    for user_entry in fs::read_dir(base_folder)? {
        let user_entry = user_entry?;
        let user_path = user_entry.path();

        if !user_path.is_dir() {
            continue;
        }

        let info_path = user_path.join("user_info.toml");
        let video_folder = user_path.join("video");

        if !info_path.exists() || !video_folder.exists() {
            continue;
        }

        let Ok(text) = fs::read_to_string(&info_path) else {
            continue;
        };

        let Ok(user_info) = toml::from_str::<UserInfo>(&text) else {
            eprintln!("failed to parse: {}", info_path.display());
            continue;
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

                if !is_video_valid(&output_video) {
                    tasks.push((
                        video_path,
                        output_video,
                        user_info.display_name.clone().into_owned(),
                        user_info.username.clone().into_owned(),
                    ));
                }
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

    for (video_path, output_video, display_name, username) in tasks {
        // Combine the display name and username with a newline
        let raw_text = format!("{}\n{}", display_name, username);

        let escaped_text = raw_text
            .replace('\\', "\\\\")
            .replace(':', "\\:")
            .replace('\'', "\\'")
            .trim()
            .to_string();

        let font_path = font_file
            .to_string_lossy()
            .to_string()
            .replace('\\', "\\\\");

        let font_size = 34;
        let line_spacing = 6;

        let padding_right = 50;
        let padding_bottom = 40;

        let drawtext_filter = format!(
            "drawtext=\
fontfile='{font_path}':\
text='{escaped_text}':\
fontcolor=white@0.75:\
fontsize={font_size}:\
x=w-(tw+{padding_right}):\
y=h-(th+{padding_bottom}):\
line_spacing={line_spacing}:\
text_align=2"
        );

        let drawtext_and_scale_filter = format!(
            "scale=1920:1080:force_original_aspect_ratio=decrease,\
pad=1920:1080:(ow-iw)/2:(oh-ih)/2,\
{drawtext_filter}"
        );

        let filename = video_path.file_name().map_or_else(
            || "Unknown File".to_string(),
            |name| name.to_string_lossy().into_owned(),
        );

        pb.set_message(format!("Encoding {filename}"));

        let error_log_path = output_video.with_extension("mp4.error.log");

        let video_path_str = video_path.to_string_lossy();
        let output_video_str = output_video.to_string_lossy();

        let crf_str = crf.map(|c| c.to_string());

        let mut args = vec![
            "-y",
            "-i",
            video_path_str.as_ref(),
            "-vf",
            drawtext_and_scale_filter.as_str(),
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-r",
            "30",
            "-map",
            "0:v:0",
            "-map",
            "0:a?",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
        ];

        if let Some(ref crf_val) = crf_str {
            args.push("-crf");
            args.push(crf_val.as_str());
        }

        args.push(output_video_str.as_ref());

        let mut child = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn ffmpeg")?;

        let mut stderr_output = String::new();

        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr);
            reader.read_to_string(&mut stderr_output)?;
        }

        let status = child.wait()?;

        if !status.success() {
            eprintln!("FFmpeg failed for '{}'", video_path.display());
            fs::write(&error_log_path, stderr_output)?;
        }

        pb.inc(1);
    }

    pb.finish_with_message("All videos processed 🎬");

    Ok(())
}
