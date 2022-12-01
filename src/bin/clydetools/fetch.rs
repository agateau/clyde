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
use crate::gh_fetcher::{gh_fetch, is_hosted_on_github};

#[derive(Debug)]
pub enum UpdateStatus {
    NoFetcher,
    UpToDate,
    NeedUpdate {
        version: Version,
        urls: HashMap<ArchOs, String>,
    },
}

fn fetch_update(ui: &Ui, package: &Package) -> Result<UpdateStatus> {
    if is_hosted_on_github(package)? {
        return gh_fetch(ui, package);
    }
    Ok(UpdateStatus::NoFetcher)
}

pub fn fetch(app: &App, ui: &Ui, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let package = Package::from_file(path)?;
        ui.info(&format!("Fetching updates for {}", package.name));
        let ui2 = ui.nest();
        let fetch_status = match fetch_update(&ui2, &package) {
            Ok(x) => x,
            Err(x) => {
                ui2.error(&format!("Could not fetch updates: {}", x));
                continue;
            }
        };

        let (version, urls) = match fetch_status {
            UpdateStatus::NoFetcher => {
                ui2.info("Don't know how to fetch updates for this package");
                continue;
            }
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
