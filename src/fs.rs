use std::path::Path;
use tokio::fs;
use tokio::io;
use anyhow::{Context, Result};

use crate::config::Config;

pub async fn ensure_out_dir_exists(config: &Config) -> Result<()> {
    let path = &config.fs.out_dir;

    tokio::fs::create_dir_all(&path)
        .await
        .context("failed to create the output directory")?;

    Ok(())
}

pub async fn clean_burned_dirs(base_folder: &Path) -> io::Result<()> {
    let mut level1 = fs::read_dir(base_folder).await?;

    while let Some(entry1) = level1.next_entry().await? {
        let path1 = entry1.path();
        if !path1.is_dir() {
            continue;
        }

        let mut level2 = fs::read_dir(&path1).await?;

        while let Some(entry2) = level2.next_entry().await? {
            let path2 = entry2.path();
            if !path2.is_dir() {
                continue;
            }

            let mut level3 = fs::read_dir(&path2).await?;

            while let Some(entry3) = level3.next_entry().await? {
                let path3 = entry3.path();

                if path3.is_dir()
                    && path3
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| name == "burned")
                {
                    fs::remove_dir_all(&path3).await?;
                }
            }
        }
    }

    Ok(())
}