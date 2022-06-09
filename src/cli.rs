use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::env;
use std::path::Path;

use crate::app::App;
use crate::install::install;
use crate::show::show;

/// A dumb application package manager
#[derive(Debug, Parser)]
#[clap(name = "pinky", version)]
pub struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
        let home_var = env::var("HOME")?;
        let prefix = Path::new(&home_var).join(".local");

        let app = App::new(&prefix);
        match self.command {
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
