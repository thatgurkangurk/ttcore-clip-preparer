use crate::burner;
use crate::config::Config;
use anyhow::{Context, Result};

pub fn burn_credits_cmd(config: &Config, video_id: String, crf: Option<i32>) -> Result<()> {
    burner::burn_multiline_text_batch(&config.fs.out_dir.join(video_id), &config.fs.font_file, crf)
        .context("failed to burn credits text")?;

    Ok(())
}
