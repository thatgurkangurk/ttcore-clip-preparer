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
    pub fn load() -> Result<Config, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("CONFIG").separator("_"))
            .build()?;

        settings.try_deserialize()
    }
}
