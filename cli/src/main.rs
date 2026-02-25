use anyhow::Result;
use clap::Parser;

use cli_template::{args::Cli, commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(cli)
}
