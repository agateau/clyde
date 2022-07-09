use std::fs;
use std::include_str;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::app::App;

fn create_activate_script(app: &App) -> Result<()> {
    let install_dir = app.install_dir.to_str().unwrap();
    let content = format!(include_str!("activate.sh.tmpl"), install_dir = install_dir);

    let scripts_dir = app.home.join("scripts");
    let script_path = scripts_dir.join("activate.sh");
    eprintln!("Creating activation script");

    fs::create_dir(&scripts_dir)?;
    fs::write(&script_path, &content)?;

    eprintln!("\nAll set! To activate your Clyde installation, add this line to your shell startup script:\n\n\
              . {script_path:?}");
    Ok(())
}

pub fn setup(home: &Path) -> Result<()> {
    if home.exists() {
        return Err(anyhow!("Clyde directory ({:?}) already exists, not doing anything. Delete it if you want to start over.",
            home));
    }
    eprintln!("Setting up Clyde in {:?}", home);

    fs::create_dir_all(&home)?;

    let app = App::new(home)?;

    app.store.setup()?;

    eprintln!("Creating Clyde database");
    app.database.create()?;

    create_activate_script(&app)?;

    Ok(())
}
