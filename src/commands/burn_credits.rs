use crate::burner::credits::burn_credits;
use crate::config::Config;
use anyhow::{Context, Result};

pub fn burn_credits_cmd(config: &Config, video_id: String, crf: Option<i32>) -> Result<()> {
    burn_credits(&config.fs.out_dir.join(video_id), &config.fs.font_file, crf)
        .context("failed to burn credits text")?;

    Ok(())
}
