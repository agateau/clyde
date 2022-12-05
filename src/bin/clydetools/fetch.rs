// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use semver::Version;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::package::{FetcherConfig, Package, Release};
use clyde::ui::Ui;

use crate::add_assets::add_asset;
use crate::github_fetcher::GitHubFetcher;
use crate::gitlab_fetcher::GitLabFetcher;

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
    fn can_fetch(&self, package: &Package) -> bool;
    fn fetch(&self, ui: &Ui, package: &Package) -> Result<UpdateStatus>;
}

fn find_fetcher<'a>(
    fetchers: &'a HashMap<FetcherConfig, Box<dyn Fetcher>>,
    package: &Package,
) -> Option<&'a dyn Fetcher> {
    match &package.fetcher {
        FetcherConfig::Auto => fetchers
            .values()
            .find(|&x| x.can_fetch(package))
            .map(|x| &**x),
        FetcherConfig::Off => None,
        fetcher_config => match fetchers.get(fetcher_config) {
            Some(x) => Some(&**x),
            None => None,
        },
    }
}

pub fn fetch(app: &App, ui: &Ui, paths: &[PathBuf]) -> Result<()> {
    let ghf: Box<dyn Fetcher> = Box::<GitHubFetcher>::default();
    let glf: Box<dyn Fetcher> = Box::<GitLabFetcher>::default();
    let fetchers = HashMap::<FetcherConfig, Box<dyn Fetcher>>::from([
        (FetcherConfig::GitHub, ghf),
        (FetcherConfig::GitLab, glf),
    ]);

    for path in paths {
        let package = Package::from_file(path)?;
        ui.info(&format!("Fetching updates for {}", package.name));
        let ui2 = ui.nest();
        let fetcher = match find_fetcher(&fetchers, &package) {
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
