use anyhow::Result;
use clap::Parser;

use clyde::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.exec()
}
