// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header;
use semver::Version;
use serde_json::{self, Value};

use clyde::app::App;
use clyde::package::Package;
use clyde::ui::Ui;

use crate::add_build::add_builds;

/// Extract GitHub `<repo>/<owner>` for a package, if it's hosted on GitHub.
/// Returns None if it's not.
///
/// Can be simplified once https://github.com/agateau/clyde/issues/67 is done and all packages have
/// been updated.
fn get_repo_owner(package: &Package) -> Result<Option<String>> {
    let latest_version = package
        .get_latest_version()
        .ok_or_else(|| anyhow!("Can't get latest version of {}", package.name))?;

    let build = package.releases[latest_version]
        .values()
        .next()
        .ok_or_else(|| anyhow!("No build available for version {}", latest_version))?;

    let rx = Regex::new("https://github.com/(?P<repo_owner>([^/]+)/([^/]+))").unwrap();

    let repo_owner = rx
        .captures(&build.url)
        .map(|captures| captures["repo_owner"].to_string());
    Ok(repo_owner)
}

/// Query GitHub REST API for the latest release, store the response in `release_file`
fn get_latest_release(release_file: &Path, repo_owner: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{repo_owner}/releases/latest");

    let client = Client::new();

    let mut response = client
        .get(url)
        .header(header::USER_AGENT, "clydetools")
        .send()?;
    if !response.status().is_success() {
        let code = response.status().as_u16();
        let body = response
            .text()
            .unwrap_or_else(|_| "[unreadable]".to_string());
        return Err(anyhow!("Request failed with error {code}:\n{body}"));
    }
    let mut file = File::create(&release_file)?;
    response.copy_to(&mut file)?;
    Ok(())
}

fn get_release_json(ui: &Ui, out_dir: &Path, package: &Package, repo_owner: &str) -> Result<Value> {
    let release_file = out_dir.join(format!("{}.json", package.name));
    if !release_file.exists() {
        ui.info("Querying GitHub");
        get_latest_release(&release_file, repo_owner)?;
    }
    let file = File::open(release_file)?;
    let json = serde_json::from_reader(&file)?;
    Ok(json)
}

fn count_chars(txt: &str, wanted: char) -> u32 {
    let mut count = 0;
    for ch in txt.chars() {
        if ch == wanted {
            count += 1;
        }
    }
    count
}

/// Extract the version of the release by parsing the `tag_name` field
fn extract_version(value: &Value) -> Result<Version> {
    let tag = value["tag_name"]
        .as_str()
        .expect("No 'tag_name' in release JSON");
    let version_str = if tag.starts_with('v') {
        tag.get(1..).unwrap()
    } else {
        tag
    };

    let mut version_str = version_str.to_string();

    while count_chars(&version_str, '.') < 2 {
        version_str.push_str(".0");
    }

    let version = Version::parse(&version_str)?;
    Ok(version)
}

fn extract_build_urls(value: &Value) -> Result<Vec<String>> {
    let assets = value["assets"]
        .as_array()
        .expect("No 'assets' element in release JSON");
    let urls = assets
        .iter()
        .map(|asset| {
            asset["browser_download_url"]
                .as_str()
                .expect("No 'browser_download_url' in release JSON asset")
                .to_string()
        })
        .collect();
    Ok(urls)
}

fn gh_update_package(app: &App, ui: &Ui, out_dir: &Path, package_path: &Path) -> Result<()> {
    let package = Package::from_file(package_path)?;

    let repo_owner = match get_repo_owner(&package)? {
        Some(x) => x,
        None => {
            ui.warn(&format!(
                "Not updating {}: not hosted on GitHub",
                package.name
            ));
            return Ok(());
        }
    };
    ui.info(&format!(
        "{} is hosted on GitHub ({repo_owner})",
        package.name
    ));

    let ui = ui.nest();

    let release_json = get_release_json(&ui, out_dir, &package, &repo_owner)?;
    let github_latest_version = extract_version(&release_json)?;

    let package_latest_version = package
        .get_latest_version()
        .ok_or_else(|| anyhow!("Can't get latest version of {}", package.name))?;

    if package_latest_version >= &github_latest_version {
        ui.info("Package is up-to-date");
        return Ok(());
    }

    ui.info(&format!("Need update to {}", github_latest_version));

    let urls = extract_build_urls(&release_json)?;

    add_builds(
        app,
        &ui,
        package_path,
        &github_latest_version,
        &None, /* arch_os */
        &urls,
    )
}

pub fn gh_update(app: &App, ui: &Ui, paths: &[PathBuf]) -> Result<()> {
    let out_dir = Path::new("out");
    if !out_dir.exists() {
        ui.info(&format!("Creating {} dir", out_dir.display()));
        fs::create_dir(&out_dir)
            .with_context(|| format!("Cannot create {} dir", out_dir.display()))?;
    }

    for path in paths {
        gh_update_package(app, ui, out_dir, path)?;
    }
    Ok(())
}
