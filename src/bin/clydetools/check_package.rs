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
use clyde::file_utils::get_file_name;
use clyde::package::Package;
use clyde::store::INDEX_NAME;
use clyde::ui::Ui;

fn check_has_release_assets(package: &Package) -> Result<()> {
    if package.releases.is_empty() {
        return Err(anyhow!("No releases"));
    }
    for (version, release) in package.releases.iter() {
        if release.is_empty() {
            return Err(anyhow!("No release assets for version {}", version));
        }
    }
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
        .get_asset(version, &ArchOs::current())
        .map(|_| version.clone())
}

fn check_can_install(ui: &Ui, package_path: &Path, version: &Version) -> Result<()> {
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

    ui.info(&format!("Executing {cmd:?}"));
    let status = cmd.status().context("Failed to execute command")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(x) => Err(anyhow!("Command failed with exit code {x}")),
        None => Err(anyhow!("Command terminated by signal")),
    }
}

fn check_package_name(package: &Package, path: &Path) -> Result<()> {
    let file_name = get_file_name(path)?;
    let package_file_name = if file_name == INDEX_NAME {
        get_file_name(path.parent().unwrap())?
    } else {
        match file_name.rsplit_once('.') {
            Some((stem, _ext)) => stem,
            None => {
                return Err(anyhow!("Invalid package name ({})", path.display()));
            }
        }
    };
    if package.name != package_file_name {
        return Err(anyhow!(
            "Package name ({}) must match the package file name ({})",
            package.name,
            package_file_name
        ));
    }
    Ok(())
}

/// The bool indicates if an asset was available
fn check_package(ui: &Ui, path: &Path) -> Result<bool> {
    let path = path.canonicalize()?;
    let package = Package::from_file(&path)?;

    check_package_name(&package, &path)?;
    check_has_release_assets(&package)?;
    check_has_installs(&package)?;

    let version = match get_latest_version(&package) {
        Some(x) => x,
        None => {
            ui.info(&format!(
                "No release assets available for {}",
                ArchOs::current()
            ));
            return Ok(false);
        }
    };
    check_can_install(ui, &path, &version)?;
    Ok(true)
}

fn print_summary_line(header: &str, packages: &[&str]) {
    let joined = packages.join(", ");
    println!("{}: {}", header, joined);
}

pub fn check_packages(ui: &Ui, paths: &Vec<PathBuf>) -> Result<()> {
    let mut ok_packages = Vec::<&str>::new();
    let mut not_on_arch_os_packages = Vec::<&str>::new();
    let mut failed_packages = Vec::<&str>::new();

    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap();
        ui.info(&format!("Checking {name}"));
        let ui2 = ui.nest();
        match check_package(&ui2, path) {
            Ok(true) => ok_packages.push(name),
            Ok(false) => not_on_arch_os_packages.push(name),
            Err(message) => {
                ui2.error(&format!("Error: {message}"));
                failed_packages.push(name)
            }
        };
    }

    ui.info("Finished");
    print_summary_line("OK   ", &ok_packages);
    print_summary_line("N/A  ", &not_on_arch_os_packages);
    print_summary_line("FAIL ", &failed_packages);

    if !failed_packages.is_empty() {
        return Err(anyhow!("{} package(s) failed", failed_packages.len()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clyde::package::Package;

    #[test]
    fn check_has_release_assets_fails_if_a_release_has_no_assets() {
        // GIVEN a package with a release containing no assets
        let package = Package::from_yaml_str(
            "
        name: test
        description: desc
        homepage:
        releases:
            1.2.0:
        ",
        )
        .unwrap();

        // WHEN check_has_release_assets() is called
        let result = check_has_release_assets(&package);

        // THEN it fails
        assert!(result.is_err());
    }

    #[test]
    fn check_has_release_assets_fails_if_it_has_no_releases() {
        // GIVEN a package with no release
        let package = Package::from_yaml_str(
            "
        name: test
        description: desc
        homepage:
        releases:
        ",
        )
        .unwrap();

        // WHEN check_has_release_assets() is called
        let result = check_has_release_assets(&package);

        // THEN it fails
        assert!(result.is_err());
    }
}
