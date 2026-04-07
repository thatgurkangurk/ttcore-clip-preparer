use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use super::consts::{
    BASE_SCALE_FILTER, FONT_SIZE, LINE_SPACING, LINE_STAGGER, OUTRO_LINE_1, OUTRO_LINE_2,
    PADDING_BOTTOM, SLIDE_DUR, SWITCH_TIME,
};
use super::utils::{generate_line_filter, get_video_duration_sec, run_ffmpeg_filter};
use crate::config::Config;

#[derive(Args, Debug, Clone)]
pub struct OutroTextArgs {
    #[arg(short, long)]
    pub input: PathBuf,
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub display_name: String,
    #[arg(long)]
    pub username: String,
    /// optional seconds of padding before the video ends to trigger the outro sequence
    #[arg(long, default_value_t = 0.0)]
    pub padding: f64,
}

pub fn process_outro_text(args: &OutroTextArgs, config: &Config) -> Result<()> {
    let font_path_str = config.fs.font_file.to_string_lossy().replace('\\', "\\\\");
    let y_bottom = format!("h-({FONT_SIZE}+{PADDING_BOTTOM})");
    let y_top = format!("h-({FONT_SIZE}*2+{PADDING_BOTTOM}+{LINE_SPACING})");

    let duration_sec = get_video_duration_sec(&args.input)?;

    // Shift our timeline backward by the padding amount
    let end_anchor_sec = (duration_sec - args.padding).max(0.0);
    let switch_sec = (end_anchor_sec - SWITCH_TIME.as_secs_f64()).max(0.0);

    let slide_sec = SLIDE_DUR.as_secs_f64();
    let stagger_sec = LINE_STAGGER.as_secs_f64();

    let text_1_line_1_in = -slide_sec;
    let text_1_line_2_in = -slide_sec;
    let text_1_line_1_out = switch_sec - slide_sec - stagger_sec;
    let text_1_line_2_out = switch_sec - slide_sec;

    let text_2_line_1_in = switch_sec;
    let text_2_line_2_in = switch_sec + stagger_sec;

    let text_2_line_1_out =
        (end_anchor_sec - slide_sec - stagger_sec).max(text_2_line_1_in + slide_sec);
    let text_2_line_2_out = (end_anchor_sec - slide_sec).max(text_2_line_2_in + slide_sec);

    let filter_text_1_line_1 = generate_line_filter(
        &args.display_name,
        &font_path_str,
        &y_top,
        text_1_line_1_in,
        text_1_line_1_out,
        &format!("between(t,0,{switch_sec})"),
    );

    let filter_text_1_line_2 = generate_line_filter(
        &args.username,
        &font_path_str,
        &y_bottom,
        text_1_line_2_in,
        text_1_line_2_out,
        &format!("between(t,0,{switch_sec})"),
    );

    let filter_text_2_line_1 = generate_line_filter(
        OUTRO_LINE_1,
        &font_path_str,
        &y_top,
        text_2_line_1_in,
        text_2_line_1_out,
        &format!("gt(t,{text_2_line_1_in})"),
    );

    let filter_text_2_line_2 = generate_line_filter(
        OUTRO_LINE_2,
        &font_path_str,
        &y_bottom,
        text_2_line_2_in,
        text_2_line_2_out,
        &format!("gt(t,{text_2_line_2_in})"),
    );

    let drawtext_and_scale_filter = format!(
        "{BASE_SCALE_FILTER},{filter_text_1_line_1},{filter_text_1_line_2},{filter_text_2_line_1},{filter_text_2_line_2}"
    );

    run_ffmpeg_filter(&args.input, &args.output, &drawtext_and_scale_filter)
}
