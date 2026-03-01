use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
        video_id: i32,
    },
    BurnCredits {
        #[arg(long)]
        video_id: i32,
    },
    Clean,
    CleanBurned,
    Update,
}
