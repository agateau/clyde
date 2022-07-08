use std::path::Path;

use anyhow::{anyhow, Result};

use clyde::package::Package;

fn check_at_least_one_release(package: &Package) -> Result<()> {
    package
        .get_latest_version()
        .ok_or_else(|| anyhow!("No version defined"))?;
    Ok(())
}

fn check_installs(package: &Package) -> Result<()> {
    if package.installs.is_empty() {
        return Err(anyhow!("No installs"));
    }
    for (version, installs_for_arch_os) in package.installs.iter() {
        if installs_for_arch_os.is_empty() {
            return Err(anyhow!("No install for version {}", version));
        }
    }
    Ok(())
}

pub fn check_package(path: &Path) -> Result<()> {
    let package = Package::from_file(path)?;

    check_at_least_one_release(&package)?;
    check_installs(&package)?;

    Ok(())
}
