use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::BufReader;
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

        let font_size = 34;
        let line_spacing = 6;

        let padding_right = 50;
        let padding_bottom = 40;

        let drawtext_filter = format!(
            "drawtext=\
fontfile='{}':\
text='{}':\
fontcolor=white@0.75:\
fontsize={}:\
x=w-(tw+{}):\
y=h-(th+{}):\
line_spacing={}:\
text_align=2",
            font_path, escaped_text, font_size, padding_right, padding_bottom, line_spacing
        );

        let drawtext_and_scale_filter = format!(
            "scale=1920:1080:force_original_aspect_ratio=decrease,\
pad=1920:1080:(ow-iw)/2:(oh-ih)/2,\
{}",
            drawtext_filter
        );

        pb.set_message(format!(
            "Encoding {}",
            video_path.file_name().unwrap().to_string_lossy()
        ));

        use std::io::Read;

        let error_log_path = output_video.with_extension("mp4.error.log");

        let mut child = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &video_path.to_string_lossy(),
                "-vf",
                &drawtext_and_scale_filter,
                "-c:v",
                "libx264",
                "-preset",
                "fast",
                "-crf",
                "23",
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
                &output_video.to_string_lossy(),
            ])
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
            eprintln!("FFmpeg failed for {:?}", video_path);
            fs::write(&error_log_path, stderr_output)?;
        }

        pb.inc(1);
    }

    pb.finish_with_message("All videos processed 🎬");

    Ok(())
}
