use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct IntroTextArgs {
    #[arg(long)]
    pub clip_id: String,
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct OutroTextArgs {
    #[arg(short, long)]
    pub clip_id: String,
    #[arg(short, long)]
    pub output: PathBuf,
    /// optional seconds of padding before the video ends to trigger the outro sequence
    #[arg(long, default_value_t = 0.0)]
    pub padding: f64,
}

#[derive(Args, Debug, Clone)]
pub struct BurnSingleClipArgs {
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(long)]
    pub clip_id: String,
}
