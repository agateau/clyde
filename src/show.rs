use anyhow::Result;

use crate::app::App;

pub fn show(app: &App, app_name: &str) -> Result<()> {
    let package = app.store.get_package(app_name)?;
    println!("Name: {}", package.name);
    println!("Description: {}", package.description);

    println!("Available versions:");
    for release in package.releases {
        println!("- {}", release.version);
    }
    Ok(())
}
