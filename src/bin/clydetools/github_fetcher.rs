// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::{self, Value};

use clyde::package::{FetcherConfig, Package};
use clyde::ui::Ui;

use crate::add_assets::select_best_urls;
use crate::fetch::{Fetcher, UpdateStatus};
use crate::version_utils::version_from_tag;

#[derive(Default)]
pub struct GitHubFetcher {}

impl Fetcher for GitHubFetcher {
    fn can_fetch(&self, package: &Package) -> bool {
        let repo_owner = get_repo_owner(package);
        repo_owner.is_some()
    }

    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus> {
        let out_dir = Path::new("out");
        if !out_dir.exists() {
            ui.info(&format!("Creating {} dir", out_dir.display()));
            fs::create_dir(out_dir)
                .with_context(|| format!("Cannot create {} dir", out_dir.display()))?;
        }

        let repo_owner = match get_repo_owner(package) {
            Some(x) => x,
            None => panic!("gh_fetch() should not be called on a package not hosted on GitHub"),
        };

        let release_json = get_release_json(ui, out_dir, package, &repo_owner)?;
        let tag = release_json["tag_name"]
            .as_str()
            .expect("No 'tag_name' in release JSON");
        let github_latest_version = version_from_tag(tag)?;

        let package_latest_version = package
            .get_latest_version()
            .ok_or_else(|| anyhow!("Can't get latest version of {}", package.name))?;

        if package_latest_version >= &github_latest_version {
            return Ok(UpdateStatus::UpToDate);
        }

        let (default_arch, default_os) = match &package.fetcher {
            FetcherConfig::GitHub { arch, os } => {
                (arch.as_ref().map(|x| &**x), os.as_ref().map(|x| &**x))
            }
            _ => (None, None),
        };

        let urls = select_best_urls(
            ui,
            &extract_build_urls(&release_json)?,
            default_arch,
            default_os,
        )?;

        Ok(UpdateStatus::NeedUpdate {
            version: github_latest_version,
            urls,
        })
    }
}

/// Extract GitHub `<repo>/<owner>` for a package, if it's hosted on GitHub.
/// Returns None if it's not.
fn get_repo_owner(package: &Package) -> Option<String> {
    let rx = Regex::new("https://github.com/(?P<repo_owner>([^/]+)/([^/]+))").unwrap();

    rx.captures(&package.repository)
        .map(|captures| captures["repo_owner"].to_string())
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
    let mut file = File::create(release_file)?;
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
