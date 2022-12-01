// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header;
use semver::Version;
use serde_json::{self, Value};

use clyde::package::Package;
use clyde::ui::Ui;

use crate::add_assets::select_best_urls;
use crate::fetch::UpdateStatus;

/// Extract GitLab `<repo>/<owner>` for a package, if it's hosted on GitLab.
/// Returns None if it's not.
fn get_repo_owner(package: &Package) -> Result<Option<String>> {
    let rx = Regex::new("https://gitlab.com/(?P<repo_owner>([^/]+)/([^/]+))").unwrap();

    let repo_owner = rx
        .captures(&package.repository)
        .map(|captures| captures["repo_owner"].to_string());
    Ok(repo_owner)
}

/// Query GitLab REST API for the latest release, store the response in `release_file`
fn get_latest_release(release_file: &Path, repo_owner: &str) -> Result<()> {
    let id = repo_owner.replace('/', "%2F");
    let url = format!("https://gitlab.com/api/v4/projects/{id}/releases/permalink/latest");

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
    let mut file = File::create(release_file)?;
    response.copy_to(&mut file)?;
    Ok(())
}

fn get_release_json(ui: &Ui, out_dir: &Path, package: &Package, repo_owner: &str) -> Result<Value> {
    let release_file = out_dir.join(format!("{}.json", package.name));
    if !release_file.exists() {
        ui.info("Querying GitLab");
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
        .as_object()
        .expect("No 'assets' element in release JSON");
    let links = assets["links"]
        .as_array()
        .expect("No 'assets.links' element in release JSON");
    let urls = links
        .iter()
        .map(|asset| {
            asset["direct_asset_url"]
                .as_str()
                .expect("No 'direct_asset_url' in release JSON links")
                .to_string()
        })
        .collect();
    Ok(urls)
}

pub fn is_hosted_on_gitlab(package: &Package) -> Result<bool> {
    let repo_owner = get_repo_owner(package)?;
    Ok(repo_owner.is_some())
}

pub fn gitlab_fetch(ui: &Ui, package: &Package) -> Result<UpdateStatus> {
    let out_dir = Path::new("out");
    if !out_dir.exists() {
        ui.info(&format!("Creating {} dir", out_dir.display()));
        fs::create_dir(out_dir)
            .with_context(|| format!("Cannot create {} dir", out_dir.display()))?;
    }

    let repo_owner = match get_repo_owner(package)? {
        Some(x) => x,
        None => panic!("gitlab_fetch() should not be called on a package not hosted on GitLab"),
    };

    let release_json = get_release_json(ui, out_dir, package, &repo_owner)?;
    let github_latest_version = extract_version(&release_json)?;

    let package_latest_version = package
        .get_latest_version()
        .ok_or_else(|| anyhow!("Can't get latest version of {}", package.name))?;

    if package_latest_version >= &github_latest_version {
        return Ok(UpdateStatus::UpToDate);
    }

    let urls = select_best_urls(ui, &extract_build_urls(&release_json)?)?;

    Ok(UpdateStatus::NeedUpdate {
        version: github_latest_version,
        urls,
    })
}
