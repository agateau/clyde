use std::fs;
use std::include_str;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::app::App;

fn create_activate_script(app: &App) -> Result<()> {
    let install_dir = app.install_dir.to_str().unwrap();
    let content = format!(include_str!("activate.sh.tmpl"), install_dir = install_dir);

    let scripts_dir = app.prefix.join("scripts");
    let script_path = scripts_dir.join("activate.sh");
    eprintln!("Creating activation script");

    fs::create_dir(&scripts_dir)?;
    fs::write(&script_path, &content)?;

    eprintln!("\nAll set! To activate your Clyde installation, add this line to your shell startup script:\n\n\
              . {script_path:?}");
    Ok(())
}

pub fn setup(prefix: &Path) -> Result<()> {
    if prefix.exists() {
        return Err(anyhow!("Clyde prefix directory ({:?}) already exists, not doing anything. Delete it if you want to start over.",
            prefix));
    }
    eprintln!("Setting up Clyde in {:?}", prefix);

    fs::create_dir_all(&prefix)?;

    let app = App::new(prefix)?;

    app.store.setup()?;

    eprintln!("Creating Clyde database");
    app.database.create()?;

    create_activate_script(&app)?;

    Ok(())
}
