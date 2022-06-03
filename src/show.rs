use std::path::PathBuf;
use anyhow::Result;

use crate::package::Package;

pub fn show(app_name: &str) -> Result<()> {
    let package = Package::from_file(&PathBuf::from(app_name))?;
    println!("Name: {}", package.name);
    println!("Description: {}", package.description);

    println!("Available versions:");
    for release in package.releases {
        println!("- {}", release.version);
    }
    Ok(())
}