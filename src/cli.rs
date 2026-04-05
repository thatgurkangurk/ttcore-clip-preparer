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
    /// download all clips marked as selected for a specific video
    Download {
        #[arg(long)]
        video_id: String,
    },
    /// get the total number of clips for a specific video
    ClipCount {
        #[arg(long)]
        video_id: String,
    },

    /// burn credit information directly into the file
    BurnCredits {
        #[arg(long)]
        video_id: String,
        #[arg(long)]
        crf: Option<i32>,
    },
    /// clean the `out` directory (deletes everything !)
    Clean,

    /// clean the `out/burned` directory
    CleanBurned,

    /// update the tool
    Update,

    /// list all videos created on the frontend
    ListVideos,
}
