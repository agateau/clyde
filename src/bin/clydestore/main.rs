use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod add_build;
pub mod import_hermit;

#[macro_use]
extern crate lazy_static;

use add_build::add_builds;
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
        #[clap(short, long)]
        arch_os: Option<String>,
        urls: Vec<String>,
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
            urls,
        } => add_builds(&package_file, &version, &arch_os, &urls),
    }
}
