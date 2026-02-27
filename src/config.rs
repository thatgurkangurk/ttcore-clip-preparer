use serde::Deserialize;
use anyhow::{Result, Context};

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
    pub fn load() -> Result<Config> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("CONFIG").separator("_"))
            .build()
            .context("failed to load the config")?;

        settings.try_deserialize().context("failed to deserialise the config")
    }
}
