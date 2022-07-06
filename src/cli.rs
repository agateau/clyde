use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::app::App;
use crate::install::install;
use crate::list::list;
use crate::search::search;
use crate::setup::setup;
use crate::show::show;
use crate::uninstall::uninstall;
use crate::update::update;
use crate::upgrade::upgrade;

/// A package manager for prebuilt applications
#[derive(Debug, Parser)]
#[clap(name = "clyde", version)]
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
        /// List application files
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
        let prefix = App::find_prefix()?;

        match self.command {
            Command::Setup {} => setup(&prefix),
            Command::Update {} => {
                let app = App::new(&prefix)?;
                update(&app)
            }
            Command::Install { package_name } => {
                let app = App::new(&prefix)?;
                install(&app, &package_name)
            }
            Command::Uninstall { package_name } => {
                let app = App::new(&prefix)?;
                uninstall(&app, &package_name)
            }
            Command::Show { package_name, list } => {
                let app = App::new(&prefix)?;
                show(&app, &package_name, list)
            }
            Command::Search { query } => {
                let app = App::new(&prefix)?;
                search(&app, &query)
            }
            Command::List {} => {
                let app = App::new(&prefix)?;
                list(&app)
            }
            Command::Upgrade {} => {
                let app = App::new(&prefix)?;
                upgrade(&app)
            }
        }
    }
}
