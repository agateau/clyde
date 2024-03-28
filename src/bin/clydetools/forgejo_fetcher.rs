// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
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

use crate::add_assets::{select_best_urls, BestUrlOptions};
use crate::fetch::{Fetcher, UpdateStatus};
use crate::version_utils::version_from_tag;

#[derive(Default)]
pub struct ForgejoFetcher {}

impl ForgejoFetcher {
    fn get_repo_owner(&self, package: &Package) -> Option<String> {
        let base_url = get_base_url(package);
        let rx = Regex::new(&format!("{base_url}/(?P<repo_owner>([^/]+)/([^/]+))")).unwrap();

        rx.captures(&package.repository)
            .map(|captures| captures["repo_owner"].to_string())
    }

    fn get_release_json(
        &self,
        ui: &Ui,
        out_dir: &Path,
        package: &Package,
        repo_owner: &str,
    ) -> Result<Value> {
        let release_file = out_dir.join(format!("{}.json", package.name));
        if !release_file.exists() {
            let base_url = get_base_url(package);
            ui.info(&format!("Querying {base_url}"));
            self.get_latest_release(&release_file, &base_url, repo_owner)?;
        }
        let file = File::open(release_file)?;
        let json: Vec<Value> = serde_json::from_reader(&file)?;

        // The JSON file is a one-item array because forgejo does not have the releases/latest
        // endpoint. Return only the item.
        let json = &json[0];
        Ok(json.clone())
    }

    /// Query the REST API for the latest release, store the response in `release_file`
    fn get_latest_release(
        &self,
        release_file: &Path,
        base_url: &str,
        repo_owner: &str,
    ) -> Result<()> {
        let url = format!("{base_url}/api/v1/repos/{repo_owner}/releases?limit=1");

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
}

fn get_base_url(package: &Package) -> String {
    match &package.fetcher {
        FetcherConfig::Forgejo { base_url, .. } => base_url.to_string(),
        _ => {
            panic!("Forgejo config must include a `base_url` key");
        }
    }
}

impl Fetcher for ForgejoFetcher {
    fn can_fetch(&self, package: &Package) -> bool {
        let repo_owner = self.get_repo_owner(package);
        repo_owner.is_some()
    }

    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus> {
        let out_dir = Path::new("out");
        if !out_dir.exists() {
            ui.info(&format!("Creating {} dir", out_dir.display()));
            fs::create_dir(out_dir)
                .with_context(|| format!("Cannot create {} dir", out_dir.display()))?;
        }

        let repo_owner = match self.get_repo_owner(package) {
            Some(x) => x,
            None => panic!("should not be called on a package not hosted on Forgejo"),
        };

        let release_json = self.get_release_json(ui, out_dir, package, &repo_owner)?;
        let tag = release_json["tag_name"]
            .as_str()
            .expect("No 'tag_name' in release JSON");
        let forgejo_latest_version = version_from_tag(tag)?;

        let package_latest_version = package
            .get_latest_version()
            .ok_or_else(|| anyhow!("Can't get latest version of {}", package.name))?;

        if package_latest_version >= &forgejo_latest_version {
            return Ok(UpdateStatus::UpToDate);
        }

        let urls = select_best_urls(
            ui,
            &extract_build_urls(&release_json)?,
            BestUrlOptions::try_from(&package.fetcher)?,
        )?;

        Ok(UpdateStatus::NeedUpdate {
            version: forgejo_latest_version,
            urls,
        })
    }
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
