mod api;
mod burner;
mod cli;
mod commands;
mod config;
mod download;
mod fs;

use clap::Parser;

use anyhow::Result;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        crate::commands::execute(command, cli.config).await?;
    }

    Ok(())
}
