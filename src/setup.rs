use std::fs;
use std::include_str;

use anyhow::{anyhow, Result};

use crate::app::App;

fn create_activate_script(app: &App) -> Result<()> {
    let install_dir = app.install_dir.to_str().unwrap();
    let content = format!(include_str!("activate.sh.tmpl"), install_dir = install_dir);

    let scripts_dir = app.prefix.join("scripts");
    let script_path = scripts_dir.join("activate.sh");
    println!("Creating activation script");

    fs::create_dir(&scripts_dir)?;
    fs::write(&script_path, &content)?;

    println!("\nAll set! To activate your Clyde installation, add this line to your shell startup script:\n\n\
              . {script_path:?}");
    Ok(())
}

pub fn setup(app: &App) -> Result<()> {
    if app.prefix.exists() {
        return Err(anyhow!("Clyde prefix directory ({:?}) already exists, not doing anything. Delete it if you want to start over.",
            app.prefix));
    }
    println!("Setting up Clyde in {:?}", app.prefix);

    fs::create_dir_all(&app.prefix)?;

    app.store.setup()?;

    println!("Creating Clyde database");
    app.get_database()?.create()?;

    create_activate_script(app)?;

    Ok(())
}
