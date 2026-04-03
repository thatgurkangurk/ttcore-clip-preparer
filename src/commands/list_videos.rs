use anyhow::Result;
use colored::Colorize;
use reqwest::Client;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Columns},
};

use crate::config::Config;

#[derive(Tabled)]
struct VideoRow {
    id: String,
    title: String,
    submissions: String,
}

pub async fn handle(client: &Client, config: &Config) -> Result<()> {
    let res = crate::api::videos::fetch_videos(client, config).await?;

    let rows: Vec<VideoRow> = res
        .videos
        .into_iter()
        .map(|v| {
            let submissions = if v.submissions_open {
                "OPEN".green().bold().to_string()
            } else {
                "CLOSED".red().bold().to_string()
            };

            VideoRow {
                id: v.id,
                title: v.title,
                submissions,
            }
        })
        .collect();

    let table = Table::new(rows)
        .with(Style::modern())
        .with(Modify::new(Columns::one(0)).with(Alignment::center()))
        .with(Modify::new(Columns::one(1)).with(Alignment::left()))
        .with(Modify::new(Columns::one(2)).with(Alignment::center()))
        .clone();

    println!("{table}");

    Ok(())
}
