// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use semver::Version;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::package::{Package, Release};
use clyde::ui::Ui;

use crate::add_assets::add_asset;
use crate::github_fetcher::{is_hosted_on_github, GitHubFetcher};
use crate::gitlab_fetcher::{is_hosted_on_gitlab, GitLabFetcher};

#[derive(Debug)]
pub enum UpdateStatus {
    UpToDate,
    NeedUpdate {
        version: Version,
        urls: HashMap<ArchOs, String>,
    },
}

/// A Fetcher knows how to fetch updates for a package
pub trait Fetcher {
    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus>;
}

fn find_fetcher(package: &Package) -> Result<Option<Box<dyn Fetcher>>> {
    if is_hosted_on_gitlab(package)? {
        return Ok(Some(Box::new(GitLabFetcher::default())));
    }
    if is_hosted_on_github(package)? {
        return Ok(Some(Box::new(GitHubFetcher::default())));
    }
    Ok(None)
}

pub fn fetch(app: &App, ui: &Ui, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let package = Package::from_file(path)?;
        ui.info(&format!("Fetching updates for {}", package.name));
        let ui2 = ui.nest();
        let fetcher = match find_fetcher(&package)? {
            Some(x) => x,
            None => {
                ui2.info("Don't know how to fetch updates for this package");
                continue;
            }
        };
        let fetch_status = match fetcher.fetch(&ui2, &package) {
            Ok(x) => x,
            Err(x) => {
                ui2.error(&format!("Could not fetch updates: {}", x));
                continue;
            }
        };

        let (version, urls) = match fetch_status {
            UpdateStatus::UpToDate => {
                ui2.info("Package is up-to-date");
                continue;
            }
            UpdateStatus::NeedUpdate { version, urls } => {
                ui2.info(&format!("Package can be updated to version {}", version));
                (version, urls)
            }
        };

        if urls.is_empty() {
            ui2.error("No assets found");
            continue;
        }

        let mut release = Release::new();
        for (arch_os, url) in urls {
            add_asset(&ui2, &app.download_cache, &mut release, &arch_os, &url)?;
        }
        let new_package = package.replace_release(&version, release);
        new_package.to_file(path)?;
    }
    Ok(())
}
