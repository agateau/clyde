// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::{Path, PathBuf};
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

fn check_package_name(package: &Package, path: &Path) -> Result<()> {
    let name = path.file_stem().unwrap().to_str().unwrap();
    if package.name != name {
        return Err(anyhow!(
            "Package name ({}) must be the package file name without extension ({})",
            package.name,
            name
        ));
    }
    Ok(())
}

/// The bool indicates if a build was available
fn check_package(path: &Path) -> Result<bool> {
    let package = Package::from_file(path)?;

    check_package_name(&package, path)?;
    check_has_releases(&package)?;
    check_has_installs(&package)?;

    let version = match get_latest_version(&package) {
        Some(x) => x,
        None => {
            println!("No builds available for {}", ArchOs::current());
            return Ok(false);
        }
    };
    check_can_install(path, &version)?;
    Ok(true)
}

fn print_summary_line(header: &str, packages: &[&str]) {
    let joined = packages.join(", ");
    println!("{}: {}", header, joined);
}

pub fn check_packages(paths: &Vec<PathBuf>) -> Result<()> {
    let mut ok_packages = Vec::<&str>::new();
    let mut no_build_packages = Vec::<&str>::new();
    let mut failed_packages = Vec::<&str>::new();

    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        println!("\n# Checking {name}");
        match check_package(path) {
            Ok(true) => ok_packages.push(name),
            Ok(false) => no_build_packages.push(name),
            Err(message) => {
                println!("Error: {message}");
                failed_packages.push(name)
            }
        };
    }

    println!("\n# Summary");
    print_summary_line("OK      ", &ok_packages);
    print_summary_line("NO BUILD", &no_build_packages);
    print_summary_line("FAIL    ", &failed_packages);

    if !failed_packages.is_empty() {
        return Err(anyhow!("{} package(s) failed", failed_packages.len()));
    }
    Ok(())
}
