use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod add_build;
pub mod import_hermit;

use add_build::add_build;
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
    ImportHermit {
        package_file: String,
    },
    AddBuild {
        package_file: PathBuf,
        version: String,
        arch_os: String,
        url: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::ImportHermit { package_file } => import_hermit(&package_file),
        Command::AddBuild {
            package_file,
            version,
            arch_os,
            url,
        } => add_build(&package_file, &version, &arch_os, &url),
    }
}
