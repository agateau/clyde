use anyhow::Result;
use clap::Parser;

use pinky::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.exec()
}
