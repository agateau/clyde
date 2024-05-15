// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use anyhow::{anyhow, Result};
use semver::Version;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::checksum::compute_checksum;
use clyde::file_cache::FileCache;
use clyde::package::{Asset, Package, Release};
use clyde::ui::Ui;

use crate::url_selector::{select_best_urls, BestUrlOptions};

fn compute_url_checksum(
    ui: &Ui,
    cache: &FileCache,
    package: &Package,
    version: &Version,
    url: &str,
) -> Result<String> {
    let path = cache.download(ui, &package.name, version, url)?;
    ui.info("Computing checksum");
    compute_checksum(&path)
}

pub fn add_asset(
    ui: &Ui,
    cache: &FileCache,
    package: &Package,
    version: &Version,
    release: &mut Release,
    arch_os: &ArchOs,
    url: &str,
) -> Result<()> {
    let checksum = compute_url_checksum(ui, cache, package, version, url)?;

    let asset = Asset {
        url: url.to_string(),
        sha256: checksum,
    };

    release.insert(*arch_os, asset);

    Ok(())
}

pub fn add_assets(
    app: &App,
    ui: &Ui,
    path: &Path,
    version: &Version,
    arch_os: &Option<String>,
    urls: &[String],
) -> Result<()> {
    let package = Package::from_file(path)?;

    let mut release = match package.releases.get(version) {
        Some(x) => x.clone(),
        None => Release::new(),
    };

    if let Some(arch_os) = arch_os {
        if urls.len() > 1 {
            return Err(anyhow!("When using --arch-os, only one URL can be passed"));
        }
        let url = urls.first().unwrap();
        let arch_os = ArchOs::parse(arch_os)?;
        add_asset(
            ui,
            &app.download_cache,
            &package,
            version,
            &mut release,
            &arch_os,
            url,
        )?;
    } else {
        let urls_for_arch_os = select_best_urls(ui, urls, BestUrlOptions::default())?;
        for (arch_os, url) in urls_for_arch_os {
            ui.info(&format!("{arch_os}: {url}"));
            let result = add_asset(
                &ui.nest(),
                &app.download_cache,
                &package,
                version,
                &mut release,
                &arch_os,
                &url,
            );
            if let Err(err) = result {
                ui.error(&format!("Can't add {arch_os:?} build from {url}: {err}"));
                return Err(err);
            };
        }
    }

    let new_package = package.replace_release(version, release);
    new_package.to_file(path)?;

    Ok(())
}

/// Wraps add_assets to make it easier to use as a standalone command
pub fn add_assets_cmd(
    app: &App,
    ui: &Ui,
    path: &Path,
    version: &str,
    arch_os: &Option<String>,
    urls: &[String],
) -> Result<()> {
    let version = Version::parse(version)?;
    add_assets(app, ui, path, &version, arch_os, urls)
}
