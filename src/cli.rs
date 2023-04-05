// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::{ArgAction, Args, Parser, Subcommand};

use crate::app::App;
use crate::install::install_cmd;
use crate::list::list_cmd;
use crate::search::search_cmd;
use crate::setup::setup_cmd;
use crate::show::show_cmd;
use crate::ui::Ui;
use crate::uninstall::uninstall_cmd;
use crate::update::update_cmd;
use crate::upgrade::upgrade_cmd;

#[derive(Debug, Parser)]
#[command(name = "clyde", version, about)]
pub struct Cli {
    #[command(flatten)]
    global_opts: GlobalOpts,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Setup Clyde
    Setup {
        /// Update the activation scripts of an existing installation.
        #[arg(short, long)]
        update_scripts: bool,
        /// URL of the Git repository to use for the store.
        #[arg(long = "--url")]
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
    List {},
    /// Upgrade all installed applications, enforcing pinning
    Upgrade {},
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Verbosity level (can be specified multiple times)
    #[arg(long, short, global = true, action = ArgAction::Count)]
    verbose: usize,
}

impl Cli {
    pub fn exec(self) -> Result<()> {
        let ui = Ui::default();
        let home = App::find_home()?;

        let _instance = App::create_single_instance(&home)?;

        match self.command {
            Command::Setup {
                update_scripts,
                store_url,
            } => setup_cmd(&ui, &home, update_scripts, store_url.as_deref()),
            Command::Update {} => {
                let app = App::new(&home)?;
                update_cmd(&app, &ui)
            }
            Command::Install {
                reinstall,
                package_names,
            } => {
                let app = App::new(&home)?;
                install_cmd(&app, &ui, reinstall, &package_names)
            }
            Command::Uninstall { package_names } => {
                let app = App::new(&home)?;
                uninstall_cmd(&app, &ui, &package_names)
            }
            Command::Show { package_name, list } => {
                let app = App::new(&home)?;
                show_cmd(&app, &package_name, list)
            }
            Command::Search { query } => {
                let app = App::new(&home)?;
                search_cmd(&app, &query)
            }
            Command::List {} => {
                let app = App::new(&home)?;
                list_cmd(&app)
            }
            Command::Upgrade {} => {
                let app = App::new(&home)?;
                upgrade_cmd(&app, &ui)
            }
        }
    }
}
