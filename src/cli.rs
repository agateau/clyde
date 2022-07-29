// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::app::App;
use crate::install::install;
use crate::list::list;
use crate::search::search;
use crate::setup::setup;
use crate::show::show;
use crate::ui::Ui;
use crate::uninstall::uninstall;
use crate::update::update;
use crate::upgrade::upgrade;

#[derive(Debug, Parser)]
#[clap(name = "clyde", version, about)]
pub struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Setup Clyde
    Setup {},
    /// Update Clyde store
    Update {},
    /// Install an application
    Install {
        /// Application name, optionally suffixed with @version
        ///
        /// @version must follow Cargo's interpretation of Semantic Versioning:
        /// <https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html>
        package_name: String,
    },
    /// Uninstall an application (alias: remove)
    #[clap(alias("remove"))]
    Uninstall {
        /// Application name
        package_name: String,
    },
    /// Show details about an application
    Show {
        /// List application files instead of showing information
        #[clap(short, long)]
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
    #[clap(long, short, global = true, parse(from_occurrences))]
    verbose: usize,
}

impl Cli {
    pub fn exec(self) -> Result<()> {
        let ui = Ui::default();
        let home = App::find_home(&ui)?;

        match self.command {
            Command::Setup {} => setup(&ui, &home),
            Command::Update {} => {
                let app = App::new(&home)?;
                update(&app, &ui)
            }
            Command::Install { package_name } => {
                let app = App::new(&home)?;
                install(&app, &ui, &package_name)
            }
            Command::Uninstall { package_name } => {
                let app = App::new(&home)?;
                uninstall(&app, &ui, &package_name)
            }
            Command::Show { package_name, list } => {
                let app = App::new(&home)?;
                show(&app, &package_name, list)
            }
            Command::Search { query } => {
                let app = App::new(&home)?;
                search(&app, &query)
            }
            Command::List {} => {
                let app = App::new(&home)?;
                list(&app)
            }
            Command::Upgrade {} => {
                let app = App::new(&home)?;
                upgrade(&app, &ui)
            }
        }
    }
}
