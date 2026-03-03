mod api;
mod burner;
mod cli;
mod config;
mod download;
mod fs;
mod update;

use clap::Parser;

use anyhow::Result;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        command.execute(cli.config).await?;
    }

    Ok(())
}
