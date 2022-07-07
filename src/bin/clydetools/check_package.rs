use std::path::Path;

use anyhow::Result;

use clyde::package::Package;

pub fn check_package(path: &Path) -> Result<()> {
    // If from_file() succeeds, the package file should be valid
    Package::from_file(path)?;
    Ok(())
}
