use anyhow::{Context, Result};
use ffmpeg_sidecar::command::FfmpegCommand;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::burner::consts::{
    BASE_SCALE_FILTER, FONT_SIZE, LINE_SPACING, PADDING_BOTTOM, PADDING_RIGHT,
};
use crate::burner::utils::escape_text;
use crate::commands::ffmpeg::get_local_ffmpeg_paths;
use crate::download::UserInfo;

pub struct EncodeTask {
    pub input: PathBuf,
    pub output: PathBuf,
    pub user_info: UserInfo<'static>,
}

fn collect_tasks(base_folder: &Path) -> Result<Vec<EncodeTask>> {
    let mut tasks = Vec::new();

    for user_entry in fs::read_dir(base_folder)? {
        let user_path = user_entry?.path();

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

        let owned_user_info = UserInfo {
            user_id: Cow::Owned(user_info.user_id.into_owned()),
            display_name: Cow::Owned(user_info.display_name.into_owned()),
            username: Cow::Owned(user_info.username.into_owned()),
        };

        let burned_dir = user_path.join("burned");

        if let Ok(mut videos) = fs::read_dir(&video_folder)
            && videos.next().is_some()
        {
            fs::create_dir_all(&burned_dir)?;
        }

        for video_entry in fs::read_dir(&video_folder)? {
            let video_path = video_entry?.path();

            if !video_path.is_file() {
                continue;
            }

            let Some(filename) = video_path.file_name() else {
                continue;
            };
            let output_video = burned_dir.join(filename);

            if !is_video_valid(&output_video) {
                tasks.push(EncodeTask {
                    input: video_path,
                    output: output_video,
                    user_info: owned_user_info.clone(),
                });
            }
        }
    }

    Ok(tasks)
}

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
    let status = std::process::Command::new(get_local_ffmpeg_paths().ffprobe)
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

pub fn run_ffmpeg(task: &EncodeTask, font_file: &Path, crf: Option<i32>) -> Result<()> {
    let raw_text = format!(
        "{}\n{}",
        task.user_info.display_name, task.user_info.username
    );
    let escaped_text = escape_text(&raw_text);

    let font_path = font_file.to_string_lossy().replace('\\', "\\\\");

    let drawtext_filter = format!(
        "drawtext=\
fontfile='{font_path}':\
text='{escaped_text}':\
fontcolor=white@0.75:\
fontsize={FONT_SIZE}:\
x=w-(tw+{PADDING_RIGHT}):\
y=h-(th+{PADDING_BOTTOM}):\
line_spacing={LINE_SPACING}:\
text_align=2"
    );

    let filter_complex = format!("{BASE_SCALE_FILTER},{drawtext_filter}");

    let mut ffmpeg_cmd =
        FfmpegCommand::new_with_path(crate::commands::ffmpeg::get_local_ffmpeg_paths().ffmpeg);

    ffmpeg_cmd
        .arg("-y")
        .arg("-i")
        .arg(task.input.to_string_lossy().into_owned())
        .arg("-vf")
        .arg(filter_complex)
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("fast")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-movflags")
        .arg("+faststart")
        .arg("-r")
        .arg("30")
        .arg("-map")
        .arg("0:v:0")
        .arg("-map")
        .arg("0:a?")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("192k");

    if let Some(crf_val) = crf {
        ffmpeg_cmd.arg("-crf").arg(crf_val.to_string());
    }

    ffmpeg_cmd.arg(task.output.to_string_lossy().into_owned());

    let mut child = ffmpeg_cmd
        .as_inner_mut()
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg")?;

    let mut stderr_output = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        BufReader::new(&mut stderr).read_to_string(&mut stderr_output)?;
    }

    let status = child.wait()?;

    if !status.success() {
        let error_log_path = task.output.with_extension("mp4.error.log");
        fs::write(&error_log_path, stderr_output)?;
        anyhow::bail!("FFmpeg exited with non-zero status");
    }

    Ok(())
}

pub fn burn_credits(base_folder: &Path, font_file: &Path, crf: Option<i32>) -> Result<()> {
    let tasks = collect_tasks(base_folder)?;

    if tasks.is_empty() {
        println!("No videos to process.");
        return Ok(());
    }

    let pb = ProgressBar::new(tasks.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} ({eta}) {msg}",
        )?
        .progress_chars("##-"),
    );

    for task in tasks {
        let filename = task.input.file_name().map_or_else(
            || "Unknown".to_string(),
            |n| n.to_string_lossy().into_owned(),
        );

        pb.set_message(format!("Encoding {filename}"));

        if let Err(e) = run_ffmpeg(&task, font_file, crf) {
            eprintln!("Failed to process '{}': {e}", task.input.display());
        }

        pb.inc(1);
    }

    pb.finish_with_message("All videos processed 🎬");
    Ok(())
}
