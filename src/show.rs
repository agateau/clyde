use anyhow::Result;

use crate::app::App;

pub fn show(app: &App, app_name: &str, list: bool) -> Result<()> {
    let db = &app.database;
    let package = app.store.get_package(app_name)?;

    println!("Name: {}", package.name);
    println!("Description: {}", package.description);
    println!("Homepage: {}", package.homepage);

    if let Some(installed_version) = db.get_package_version(&package.name)? {
        println!("Installed version: {}", installed_version);
    }

    println!();
    println!("Available versions:");
    for version in package.releases.keys().rev() {
        println!("- {}", version);
    }

    if list {
        println!();
        println!("Installed files:");
        let fileset = db.get_package_files(&package.name)?;
        let mut files = Vec::from_iter(fileset);
        files.sort();
        for file in files {
            let path = app.install_dir.join(file);
            println!("- {}", path.display());
        }
    }
    Ok(())
}
