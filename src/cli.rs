use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::app::App;
use crate::install::install;
use crate::setup::setup;
use crate::show::show;

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
    /// Install an application
    Install {
        /// Application name
        package_name: String,
    },
    /// Uninstall an application
    Remove {
        /// Application name
        package_name: String,
    },
    /// Show details about an application
    Show {
        /// Application name
        package_name: String,
    },
    /// List installed applications
    List {},
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

        let app = App::new(&prefix);
        match self.command {
            Command::Setup {} => setup(&app),
            Command::Install { package_name } => install(&app, &package_name),
            Command::Remove { package_name } => {
                println!("Removing {}", package_name);
                Ok(())
            }
            Command::Show { package_name } => show(&app, &package_name),
            Command::List {} => {
                println!("Listing installed packages");
                Ok(())
            }
        }
    }
}
