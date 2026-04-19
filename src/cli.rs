use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    burner::{intro_text::IntroTextArgs, outro_text::OutroTextArgs},
    commands::burn_single_clip::BurnSingleClipArgs,
};

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
    BurnOutroText(OutroTextArgs),

    BurnSingleClip(BurnSingleClipArgs),
}

#[derive(Debug, Args)]
pub struct VideoArgs {
    #[command(subcommand)]
    pub command: VideoCommands,
}

#[derive(Debug, Subcommand)]
pub enum VideoCommands {
    /// create a new video
    Create { title: String },

    /// download all clips marked as selected for this video
    Download {
        /// the id of the video
        video_id: String,
    },

    /// get the total number of clips for this video
    ClipCount {
        /// the id of the video
        video_id: String,
    },

    /// burn credit information directly into the file
    BurnCredits {
        /// the id of the video
        video_id: String,

        #[arg(long)]
        crf: Option<i32>,
    },
}
