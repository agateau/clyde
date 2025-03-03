// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::vec::Vec;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod add_assets;
pub mod check_package;
pub mod fetch;
pub mod forgejo_fetcher;
pub mod github_fetcher;
pub mod gitlab_fetcher;
pub mod script_fetcher;
pub mod url_selector;
pub mod version_utils;

#[macro_use]
extern crate lazy_static;

use clyde::app::App;
use clyde::ui::Ui;

use add_assets::add_assets_cmd;
use check_package::check_packages;
use fetch::fetch_cmd;

/// Helper tools for Clyde package authors. These commands are not useful to use Clyde.
#[derive(Debug, Parser)]
#[command(name = "clydetools", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add assets to a package
    #[command(alias("add-build"))]
    AddAssets {
        /// Path to the package YAML file
        package_file: PathBuf,
        /// Version of the builds.
        ///
        /// If the YAML file does not already contain this version, it will be added.
        version: String,
        #[arg(short, long)]
        /// arch-os double
        ///
        /// If not set, add-assets tries to deduce it from the archive names. If
        /// set, then only one URL can be passed.
        arch_os: Option<String>,
        /// URLs of the build archives
        urls: Vec<String>,
    },
    /// Check the validity of packages: checks the YAML files has all the required entries, and
    /// check the latest asset installs (if it can be installed on the running machine)
    Check {
        /// Path where to store a JSON report of the checks
        #[arg(short, long)]
        report: Option<PathBuf>,

        /// Path to the package YAML files
        #[arg(required = true)]
        package_files: Vec<PathBuf>,
    },
    /// Fetch updates for supported packages
    Fetch {
        /// Path to the package YAML files
        #[arg(required = true)]
        package_files: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ui = Ui::default();
    let home = App::find_home()?;

    match cli.command {
        Command::AddAssets {
            package_file,
            version,
            arch_os,
            urls,
        } => {
            let app = App::new(&home)?;
            add_assets_cmd(&app, &ui, &package_file, &version, &arch_os, &urls)
        }
        // Check can run without an existing Clyde home: it creates a temporary one to test the package
        Command::Check {
            report,
            package_files,
        } => check_packages(&ui, &report, &package_files),
        Command::Fetch { package_files } => {
            let app = App::new(&home)?;
            fetch_cmd(&app, &ui, &package_files)
        }
    }
}
