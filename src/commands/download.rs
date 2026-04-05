use crate::config::Config;
use anyhow::{Context, Result};

use crate::download;

pub async fn download_command(
    video_id: String,
    config: &Config,
    client: &reqwest::Client,
) -> Result<()> {
    download::download_selected_files(&video_id, config, client)
        .await
        .context("download command failed")?;

    Ok(())
}
