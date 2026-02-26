use anyhow::Result;
use clap::Parser;
use tokio::main;

use cli_template::{args::Cli, commands};

#[main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(cli).await
}
