use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Args;

use super::consts::{
    BASE_SCALE_FILTER, FADE_DUR, FONT_SIZE, INTRO_LINE_1, INTRO_LINE_2, LINE_SPACING, LINE_STAGGER,
    PADDING_BOTTOM, PADDING_RIGHT, SLIDE_DUR, SWITCH_TIME,
};

use super::utils::escape_text;

use crate::config::Config;

#[derive(Args, Debug, Clone)]
pub struct IntroTextArgs {
    /// path to the input video
    #[arg(short, long)]
    pub input: PathBuf,

    /// path to save the output video
    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(long)]
    pub display_name: String,

    #[arg(long)]
    pub username: String,
}

pub fn process_intro_text(args: &IntroTextArgs, config: &Config) -> Result<()> {
    let font_path_str = config.fs.font_file.to_string_lossy().replace('\\', "\\\\");

    let y_bottom = format!("h-({FONT_SIZE}+{PADDING_BOTTOM})");
    let y_top = format!("h-({FONT_SIZE}*2+{PADDING_BOTTOM}+{LINE_SPACING})");

    let text_1_line_1_in = Duration::ZERO;

    let text_1_line_1_out = SWITCH_TIME
        .checked_sub(SLIDE_DUR)
        .and_then(|t| t.checked_sub(LINE_STAGGER))
        .context("SWITCH_TIME is too short")?;

    let text_1_line_2_in = LINE_STAGGER;

    let text_1_line_2_out = SWITCH_TIME
        .checked_sub(SLIDE_DUR)
        .context("SWITCH_TIME is shorter than SLIDE_DUR")?;

    let text_2_line_1_in = SWITCH_TIME;
    let text_2_line_2_in = SWITCH_TIME
        .checked_add(LINE_STAGGER)
        .context("timing overflow")?;
    let duration_max = Duration::MAX;

    let switch_sec = SWITCH_TIME.as_secs_f64();

    let filter_text_1_line_1 = generate_line_filter(
        INTRO_LINE_1,
        &font_path_str,
        &y_top,
        text_1_line_1_in,
        text_1_line_1_out,
        &format!("between(t,0,{switch_sec})"),
    );

    let filter_text_1_line_2 = generate_line_filter(
        INTRO_LINE_2,
        &font_path_str,
        &y_bottom,
        text_1_line_2_in,
        text_1_line_2_out,
        &format!("between(t,0,{switch_sec})"),
    );

    let filter_text_2_line_1 = generate_line_filter(
        &args.display_name,
        &font_path_str,
        &y_top,
        text_2_line_1_in,
        duration_max,
        &format!("gt(t,{})", text_2_line_1_in.as_secs_f64()),
    );

    let filter_text_2_line_2 = generate_line_filter(
        &args.username,
        &font_path_str,
        &y_bottom,
        text_2_line_2_in,
        duration_max,
        &format!("gt(t,{})", text_2_line_1_in.as_secs_f64()),
    );

    let drawtext_and_scale_filter = format!(
        "{BASE_SCALE_FILTER},\
    {filter_text_1_line_1},{filter_text_1_line_2},{filter_text_2_line_1},{filter_text_2_line_2}"
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &args.input.to_string_lossy(),
            "-vf",
            &drawtext_and_scale_filter,
            "-codec:a",
            "copy",
            &args.output.to_string_lossy(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn ffmpeg process")?;

    if status.success() {
        println!(
            "✅ video processed successfully! saved as {}",
            args.output.display()
        );
        Ok(())
    } else {
        anyhow::bail!("❌ ffmpeg encountered an error and exited with a non-zero status code");
    }
}

fn generate_line_filter(
    text: &str,
    font_path: &str,
    y_expr: &str,
    t_in: Duration,
    t_out: Duration,
    enable_expr: &str,
) -> String {
    let escaped_text = escape_text(text);

    let t_in_sec = t_in.as_secs_f64();
    let t_out_sec = t_out.as_secs_f64();
    let slide_sec = SLIDE_DUR.as_secs_f64();
    let fade_sec = FADE_DUR.as_secs_f64();

    let x_expr = format!(
        "w-(tw+{PADDING_RIGHT})*((1-pow(1-min(max(t-{t_in_sec},0)/{slide_sec},1),3))-pow(min(max(t-{t_out_sec},0)/{slide_sec},1),3))"
    );

    let alpha_expr =
        format!("min(max(t-{t_in_sec},0)/{fade_sec},1)-min(max(t-{t_out_sec},0)/{fade_sec},1)");

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
