use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use semver::Version;
use tempfile::TempDir;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::package::Package;

fn check_has_releases(package: &Package) -> Result<()> {
    package
        .get_latest_version()
        .ok_or_else(|| anyhow!("No version defined"))?;
    Ok(())
}

fn check_has_installs(package: &Package) -> Result<()> {
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

fn get_latest_version(package: &Package) -> Option<Version> {
    let version = package.get_latest_version().unwrap();

    package
        .get_build(version, &ArchOs::current())
        .map(|_| version.clone())
}

fn check_can_install(package_path: &Path, version: &Version) -> Result<()> {
    let home_temp_dir = TempDir::new()?;
    let home_dir = home_temp_dir.path();
    let store_dir = home_dir.join("store");
    fs::create_dir(&store_dir)
        .with_context(|| format!("Could not create store directory {store_dir:?}"))?;

    let app = App::new(home_dir).context("Could not create test Clyde home")?;
    app.database.create()?;

    let package_str = package_path.as_os_str().to_str().unwrap();
    let mut cmd = Command::new("clyde");

    cmd.env("CLYDE_HOME", home_dir.as_os_str())
        .arg("install")
        .arg(format!("{}@={}", package_str, version));

    println!("Executing {cmd:?}");
    let status = cmd.status().context("Failed to execute command")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(x) => Err(anyhow!("Command failed with exit code {x}")),
        None => Err(anyhow!("Command terminated by signal")),
    }
}

pub fn check_package(path: &Path) -> Result<()> {
    let package = Package::from_file(path)?;

    check_has_releases(&package)?;
    check_has_installs(&package)?;

    let version = match get_latest_version(&package) {
        Some(x) => x,
        None => {
            println!("No builds available for {}", ArchOs::current());
            return Ok(());
        }
    };
    check_can_install(path, &version)
}
