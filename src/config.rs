use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct FsConfig {
    pub out_dir: std::path::PathBuf,
    pub font_file: std::path::PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub fs: FsConfig,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Config> {
        let file_source = path
            .map(config::File::from)
            .unwrap_or_else(|| config::File::with_name("config"))
            .required(false);

        config::Config::builder()
            .add_source(file_source)
            .add_source(config::Environment::with_prefix("CONFIG").separator("_"))
            .build()
            .context("failed to load the config")?
            .try_deserialize()
            .context("failed to deserialize the config")
    }
}
