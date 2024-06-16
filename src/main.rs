mod command;
mod config;
mod path;
mod util;

use anyhow::Result;
use clap::Parser;
use command::Command;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    subcommand: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.subcommand.exec()
}
