use clap::Parser;
use anyhow::Result;

use pinky::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.exec()
}
