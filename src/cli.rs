use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::burner::intro_text::IntroTextArgs;

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
    /// perform operations on a specific video
    Video(VideoArgs),

    /// clean the `out` directory (deletes everything !)
    Clean,

    /// clean the `out/burned` directory
    CleanBurned,

    /// update the tool
    Update,

    /// list all videos created on the frontend
    ListVideos,

    BurnIntroText(IntroTextArgs),
}

#[derive(Debug, Args)]
pub struct VideoArgs {
    /// the id of a video
    pub video_id: String,

    #[command(subcommand)]
    pub command: VideoCommands,
}

#[derive(Debug, Subcommand)]
pub enum VideoCommands {
    /// download all clips marked as selected for this video
    Download,

    /// get the total number of clips for this video
    ClipCount,

    /// burn credit information directly into the file
    BurnCredits {
        #[arg(long)]
        crf: Option<i32>,
    },
}
