use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod add_build;
pub mod check_package;
pub mod import_hermit;

#[macro_use]
extern crate lazy_static;

use clyde::app::App;

use add_build::add_builds;
use check_package::check_package;
use import_hermit::import_hermit;

/// Helper tools to work with Clyde packages
#[derive(Debug, Parser)]
#[clap(name = "clydetools", version)]
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
    /// Check the validity of a package file
    Check {
        package_file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home = App::find_home()?;

    match cli.command {
        Command::ImportHermit { package_file } => {
            let app = App::new(&home)?;
            import_hermit(&app, &package_file)
        }
        Command::AddBuild {
            package_file,
            version,
            arch_os,
            urls,
        } => {
            let app = App::new(&home)?;
            add_builds(&app, &package_file, &version, &arch_os, &urls)
        }
        // Check can run without an existing Clyde home: it creates a temporary one to test the package
        Command::Check { package_file } => check_package(&package_file),
    }
}
