use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod import_hermit;

use import_hermit::import_hermit;

/// Helper commands to work with Clyde packages
#[derive(Debug, Parser)]
#[clap(name = "clydestore", version)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    ImportHermit { package_file: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::ImportHermit { package_file } => import_hermit(&package_file),
    }
}
