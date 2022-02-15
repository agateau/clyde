use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::install::install;

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
        app_name: String,
    },
    /// Uninstall an application
    Remove {
        /// Application name
        app_name: String,
    },
    /// List installed applications
    List {
    }
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Verbosity level (can be specified multiple times)
    #[clap(long, short, global = true, parse(from_occurrences))]
    verbose: usize,
}

impl Cli {
    pub fn exec(self) -> Result<()> {
        match self.command {
            Command::Install { app_name } => {
                install(&app_name)
            }
            Command::Remove { app_name } => {
                println!("Removing {}", app_name);
                Ok(())
            }
            Command::List {} => {
                println!("Listing installed applications");
                Ok(())
            }
        }
    }
}
