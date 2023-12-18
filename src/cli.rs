// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{Parser, Subcommand};

/// This file must build standalone because it's used by `build.rs` to generate shell
/// auto-completion files

#[derive(Debug, Parser)]
#[command(name = "clyde", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Setup Clyde
    Setup {
        /// Update the activation scripts of an existing installation.
        #[arg(short, long)]
        update_scripts: bool,
        /// URL of the Git repository to use for the store.
        #[arg(long = "url")]
        store_url: Option<String>,
    },
    /// Update Clyde store
    Update {},
    /// Install applications
    Install {
        /// Uninstall then reinstall already installed packages
        #[arg(short, long)]
        reinstall: bool,
        /// Application name, optionally suffixed with @version
        ///
        /// @version must follow Cargo's interpretation of Semantic Versioning:
        /// <https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html>
        #[arg(required = true, value_name = "APPLICATION_NAME")]
        package_names: Vec<String>,
    },
    /// Uninstall applications (alias: remove)
    #[command(alias("remove"))]
    Uninstall {
        /// Application name
        #[arg(required = true, value_name = "APPLICATION_NAME")]
        package_names: Vec<String>,
    },
    /// Show details about an application
    Show {
        /// List application files instead of showing information
        #[arg(short, long)]
        list: bool,
        /// Application name
        package_name: String,
    },
    /// Search for available applications
    Search {
        /// Search query
        query: String,
    },
    /// List installed applications
    List {
        /// Use JSON output
        #[arg(short, long)]
        json: bool,
    },
    /// Upgrade all installed applications, enforcing pinning
    Upgrade {},
}
