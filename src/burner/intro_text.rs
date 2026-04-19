use anyhow::Result;
use clap::Args;
use futures_util::StreamExt;
use std::borrow::Cow;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;

use super::consts::{
    BASE_SCALE_FILTER, FONT_SIZE, INTRO_LINE_1, INTRO_LINE_2, LINE_SPACING, LINE_STAGGER,
    PADDING_BOTTOM, SLIDE_DUR, SWITCH_TIME,
};
use super::utils::{generate_line_filter, run_ffmpeg_filter};
use crate::api::client::ApiClient;
use crate::config::Config;
use crate::download::UserInfo;

#[derive(Args, Debug, Clone)]
pub struct IntroTextArgs {
    #[arg(long)]
    pub clip_id: String,
    #[arg(short, long)]
    pub output: PathBuf,
}

pub async fn process_intro_text(
    args: &IntroTextArgs,
    config: &Config,
    api_client: &ApiClient,
) -> Result<()> {
    let temp_dir = tempdir()?;
    let res = api_client.get_single_clip(&args.clip_id).await?;

    let user_info = res.clip.overridden_profile_data.as_ref().map_or_else(
        || UserInfo {
            user_id: Cow::Owned(res.clip.creator.id.clone()),
            display_name: Cow::Owned(res.clip.creator.name.clone()),
            username: Cow::Owned(format!("@{}", res.clip.creator.username)),
        },
        |profile| UserInfo {
            user_id: Cow::Owned(format!("profile_{}", profile.id)),
            display_name: Cow::Owned(profile.line1.clone()),
            username: Cow::Owned(profile.line2.clone()),
        },
    );

    let font_path_str = config.fs.font_file.to_string_lossy().replace('\\', "\\\\");
    let y_bottom = format!("h-({FONT_SIZE}+{PADDING_BOTTOM})");
    let y_top = format!("h-({FONT_SIZE}*2+{PADDING_BOTTOM}+{LINE_SPACING})");

    let switch_sec = SWITCH_TIME.as_secs_f64();
    let slide_sec = SLIDE_DUR.as_secs_f64();
    let stagger_sec = LINE_STAGGER.as_secs_f64();

    let text_1_line_1_in = 0.0;
    let text_1_line_1_out = switch_sec - slide_sec - stagger_sec;
    let text_1_line_2_in = stagger_sec;
    let text_1_line_2_out = switch_sec - slide_sec;

    let text_2_line_1_in = switch_sec;
    let text_2_line_2_in = switch_sec + stagger_sec;
    let duration_max = 99999.0;

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
        &user_info.display_name,
        &font_path_str,
        &y_top,
        text_2_line_1_in,
        duration_max,
        &format!("gt(t,{text_2_line_1_in})"),
    );

    let filter_text_2_line_2 = generate_line_filter(
        &user_info.username,
        &font_path_str,
        &y_bottom,
        text_2_line_2_in,
        duration_max,
        &format!("gt(t,{text_2_line_2_in})"),
    );

    let drawtext_and_scale_filter = format!(
        "{BASE_SCALE_FILTER},{filter_text_1_line_1},{filter_text_1_line_2},{filter_text_2_line_1},{filter_text_2_line_2}"
    );

    let video_path = temp_dir.path().join(format!("{}.mp4", args.clip_id));

    let response = api_client
        .client
        .get(res.clip.url)
        .send()
        .await?
        .error_for_status()?;

    let mut file = tokio::fs::File::create(&video_path).await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    run_ffmpeg_filter(&video_path, &args.output, &drawtext_and_scale_filter)
}
