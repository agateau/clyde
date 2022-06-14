use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::app::App;

const CLYDE_STORE_URL: &str = "https://github.com/agateau/clyde-store";

fn setup_store(store_dir: &Path) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(["clone", CLYDE_STORE_URL]);
    cmd.arg(store_dir.as_os_str());

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("Failed to clone Clyde store"));
    }
    Ok(())
}

pub fn setup(app: &App) -> Result<()> {
    if app.prefix.exists() {
        return Err(anyhow!("Clyde prefix directory ({:?}) already exists, not doing anything. Delete it if you want to start over.",
            app.prefix));
    }
    println!("Setting up Clyde using prefix {:?}", app.prefix);

    fs::create_dir_all(&app.prefix)?;

    setup_store(&app.store_dir)?;

    Ok(())
}
