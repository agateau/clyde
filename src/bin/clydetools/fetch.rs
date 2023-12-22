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
use crate::script_fetcher::ScriptFetcher;

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

struct FetcherFinder {
    github_fetcher: GitHubFetcher,
    gitlab_fetcher: GitLabFetcher,
    script_fetcher: ScriptFetcher,
}

impl FetcherFinder {
    fn new() -> FetcherFinder {
        FetcherFinder {
            github_fetcher: GitHubFetcher::default(),
            gitlab_fetcher: GitLabFetcher::default(),
            script_fetcher: ScriptFetcher::default(),
        }
    }

    fn find(&self, package: &Package) -> Option<&dyn Fetcher> {
        let auto_fetchers: [&dyn Fetcher; 2] = [&self.github_fetcher, &self.gitlab_fetcher];

        match &package.fetcher {
            FetcherConfig::Auto => auto_fetchers
                .iter()
                .find(|&x| x.can_fetch(package))
                .map(|x| &**x),
            FetcherConfig::Off => None,
            FetcherConfig::GitHub { arch: _a, os: _o } => Some(&self.github_fetcher),
            FetcherConfig::GitLab { arch: _a, os: _o } => Some(&self.gitlab_fetcher),
            FetcherConfig::Script { .. } => Some(&self.script_fetcher),
        }
    }
}

pub fn fetch_cmd(app: &App, ui: &Ui, paths: &[PathBuf]) -> Result<()> {
    let fetcher_finder = FetcherFinder::new();

    for path in paths {
        let package = Package::from_file(path)?;
        ui.info(&format!("Fetching updates for {}", package.name));
        let ui2 = ui.nest();
        let fetcher = match fetcher_finder.find(&package) {
            Some(x) => x,
            None => {
                ui2.info("Don't know how to fetch updates for this package");
                continue;
            }
        };
        let fetch_status = match fetcher.fetch(&ui2, &package) {
            Ok(x) => x,
            Err(x) => {
                ui2.error(&format!("Could not fetch updates: {x}"));
                continue;
            }
        };

        let (version, urls) = match fetch_status {
            UpdateStatus::UpToDate => {
                ui2.info("Package is up-to-date");
                continue;
            }
            UpdateStatus::NeedUpdate { version, urls } => {
                ui2.info(&format!("Package can be updated to version {version}"));
                (version, urls)
            }
        };

        if urls.is_empty() {
            ui2.error("No assets found");
            continue;
        }

        let mut release = Release::new();
        for (arch_os, url) in urls {
            add_asset(
                &ui2,
                &app.download_cache,
                &package,
                &version,
                &mut release,
                &arch_os,
                &url,
            )?;
        }
        let new_package = package.replace_release(&version, release);
        new_package.to_file(path)?;
    }
    Ok(())
}
