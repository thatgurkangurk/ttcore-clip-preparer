use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Download {
        #[arg(long)]
        video_id: String,
    },
    ClipCount {
        #[arg(long)]
        video_id: String,
    },
    BurnCredits {
        #[arg(long)]
        video_id: String,
        #[arg(long)]
        crf: Option<i32>,
    },
    Clean,
    CleanBurned,
    Update,
    ListVideos,
}
