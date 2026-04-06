use std::{
    path::Path,
    process::{Command, Stdio},
};
use anyhow::{Context, Result};

use super::consts::{FADE_DUR, FONT_SIZE, PADDING_RIGHT, SLIDE_DUR};

pub fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "\\'")
        .trim()
        .to_string()
}

pub fn generate_line_filter(
    text: &str,
    font_path: &str,
    y_expr: &str,
    t_in_sec: f64,
    t_out_sec: f64,
    enable_expr: &str,
) -> String {
    let escaped_text = escape_text(text);
    let slide_sec = SLIDE_DUR.as_secs_f64();
    let fade_sec = FADE_DUR.as_secs_f64();

    let x_expr = format!(
        "w-(tw+{PADDING_RIGHT})*((1-pow(1-min(max(t-{t_in_sec},0)/{slide_sec},1),3))-pow(min(max(t-{t_out_sec},0)/{slide_sec},1),3))"
    );

    let alpha_expr = format!(
        "min(max(t-{t_in_sec},0)/{fade_sec},1)-min(max(t-{t_out_sec},0)/{fade_sec},1)"
    );

    format!(
        "drawtext=\
        fontfile='{font_path}':\
        text='{escaped_text}':\
        fontcolor=white@0.75:\
        fontsize={FONT_SIZE}:\
        x='{x_expr}':\
        y='{y_expr}':\
        alpha='{alpha_expr}':\
        enable='{enable_expr}'"
    )
}

pub fn run_ffmpeg_filter(input: &Path, output: &Path, filter: &str) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-vf",
            filter,
            "-codec:a",
            "copy",
            &output.to_string_lossy(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn ffmpeg process")?;

    if status.success() {
        println!("✅ video processed successfully! saved as {}", output.display());
        Ok(())
    } else {
        anyhow::bail!("❌ ffmpeg encountered an error and exited with a non-zero status code");
    }
}

pub fn get_video_duration_sec(input: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            &input.to_string_lossy()
        ])
        .output()
        .context("failed to run ffprobe")?;

    let stdout = String::from_utf8(output.stdout).context("ffprobe output invalid utf-8")?;
    stdout.trim().parse().context("failed to parse video duration")
}