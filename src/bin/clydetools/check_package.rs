// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{anyhow, Context, Result};
use semver::Version;
use shell_words;
use tempfile::TempDir;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::file_utils::{get_file_name, prepend_dir_to_path};
use clyde::package::Package;
use clyde::store::INDEX_NAME;
use clyde::ui::Ui;
use clyde::vars::{expand_vars, VarsMap};

struct FailedPackage {
    package_path: PathBuf,
    error_message: String,
}

impl FailedPackage {
    fn new(package_path: &Path, error_message: &str) -> FailedPackage {
        FailedPackage {
            package_path: package_path.to_path_buf(),
            error_message: error_message.to_string(),
        }
    }
}

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

/// Run the test command `test_command`, add results with error details to `report`
fn run_test_command(report: &mut Vec<String>, home_dir: &Path, test_command: &str) -> Result<()> {
    let clyde_bin_dir = home_dir.join("inst").join("bin");

    let new_path = prepend_dir_to_path(&clyde_bin_dir)?;

    let words = shell_words::split(test_command)?;
    let mut iter = words.iter();
    let binary = iter
        .next()
        .ok_or_else(|| anyhow!("Test command is empty"))?;
    let args: Vec<String> = iter.map(|x| x.into()).collect();

    run_command(
        report,
        Command::new(binary).env("PATH", new_path).args(args),
    )
}

fn create_vars_map() -> VarsMap {
    // Only add ${exe_ext} for now. We'll see if we need other vars in the future
    let mut map = VarsMap::new();
    map.insert(
        "exe_ext".into(),
        if cfg!(windows) {
            ".exe".into()
        } else {
            "".into()
        },
    );
    map
}

fn string_for_command_output(output: &Output) -> String {
    let mut string = String::new();
    string.push_str("--- Stdout ---\n");
    string.push_str(&String::from_utf8_lossy(&output.stdout));
    string.push_str("--- Stderr ---\n");
    string.push_str(&String::from_utf8_lossy(&output.stderr));
    string
}

/// Run `command`, add results with error details to `report`
fn run_command(report: &mut Vec<String>, command: &mut Command) -> Result<()> {
    let command_str = format!("{:?} {:?}", command.get_program(), command.get_args());
    report.push(format!("Running {command_str}"));
    let output = command.output().context("Failed to execute command")?;

    match output.status.code() {
        Some(0) => Ok(()),
        Some(x) => {
            report.push(format!("Command failed with exit code {x}"));
            report.push(string_for_command_output(&output));
            Err(anyhow!("Command failed with exit code {x}"))
        }
        None => {
            report.push("Command terminated by signal".to_string());
            Err(anyhow!("Command terminated by signal"))
        }
    }
}

/// Checks the package can install, reports failure as a big string in the Err branch of Result
fn check_can_install(package: &Package, package_path: &Path, version: &Version) -> Result<()> {
    let mut report = Vec::<String>::new();

    // Setup temp Clyde home
    report.push("### Setup Clyde home".to_string());
    let home_temp_dir = TempDir::new()?;
    let home_dir = home_temp_dir.path();
    let store_dir = home_dir.join("store");
    fs::create_dir(&store_dir)
        .with_context(|| format!("Could not create store directory {store_dir:?}"))?;

    let app = App::new(home_dir).context("Could not create test Clyde home")?;
    app.database.create()?;

    // Install package
    report.push("\n### Install package\n".to_string());
    let package_str = package_path.as_os_str().to_str().unwrap();
    run_command(
        &mut report,
        Command::new("clyde")
            .arg("install")
            .arg(format!("{package_str}@={version}"))
            .env("CLYDE_HOME", home_dir.as_os_str()),
    )
    .map_err(|_| anyhow!(report.join("\n")))?;

    // Run test commands
    report.push("\n### Running test commmands\n".to_string());
    let install = package.get_install(version, &ArchOs::current()).unwrap();

    let vars = create_vars_map();

    for test_command in &install.tests {
        let test_command = expand_vars(test_command, &vars)?;
        run_test_command(&mut report, home_dir, &test_command)
            .map_err(|_| anyhow!(report.join("\n")))?;
    }
    Ok(())
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

fn load_package(path: &Path) -> Result<Package> {
    let path = path.canonicalize()?;
    let package = Package::from_file(&path)?;
    check_package_name(&package, &path)?;
    Ok(package)
}

/// The bool indicates if an asset was available for the current arch-os
fn check_package(package: &Package, path: &Path) -> Result<bool> {
    check_has_release_assets(package)?;
    check_has_installs(package)?;

    let version = match get_latest_version(package) {
        Some(x) => x,
        None => {
            return Ok(false);
        }
    };
    check_can_install(package, path, &version)?;
    Ok(true)
}

fn print_summary_line(header: &str, packages: &[String]) {
    let joined = packages.join(", ");
    println!("{header}: {joined}");
}

pub fn check_packages(ui: &Ui, paths: &Vec<PathBuf>) -> Result<()> {
    let mut ok_packages = Vec::<String>::new();
    let mut not_on_arch_os_packages = Vec::<String>::new();
    let mut failed_packages = Vec::<FailedPackage>::new();

    let count = paths.len();
    let mut idx = 1;
    for path in paths {
        print!(
            "[{idx}/{count} {:3}%] {}: ",
            100 * idx / count,
            path.display()
        );
        io::stdout().lock().flush().unwrap_or_default();
        idx += 1;
        let package = match load_package(path) {
            Ok(x) => x,
            Err(message) => {
                failed_packages.push(FailedPackage::new(path, &message.to_string()));
                println!("FAIL");
                continue;
            }
        };
        let name = package.name.clone();
        match check_package(&package, path) {
            Ok(true) => {
                ok_packages.push(name);
                println!("OK");
            }
            Ok(false) => {
                not_on_arch_os_packages.push(name);
                println!("OK (not on arch-os)");
            }
            Err(message) => {
                failed_packages.push(FailedPackage::new(path, &message.to_string()));
                println!("FAIL");
            }
        };
    }

    ui.info("Finished");
    print_summary_line("OK", &ok_packages);
    print_summary_line("N/A", &not_on_arch_os_packages);

    if !failed_packages.is_empty() {
        println!("# Failed packages\n");
        for failed_package in &failed_packages {
            println!("## {}", failed_package.package_path.display());
            println!("\n{}\n", failed_package.error_message);
        }

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
