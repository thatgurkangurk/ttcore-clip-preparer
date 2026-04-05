use crate::{api::client::ApiClient, config::Config};
use anyhow::{Context, Result};

use crate::download;

pub async fn download_command(
    video_id: String,
    config: &Config,
    api_client: &ApiClient,
) -> Result<()> {
    download::download_selected_files(&video_id, config, api_client)
        .await
        .context("download command failed")?;

    Ok(())
}
