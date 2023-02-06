// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use semver::Version;

use crate::download::download;
use crate::ui::Ui;

pub struct FileCache {
    dir: PathBuf,
}

/// Download package assets in a persistent directory.
///
/// Assets are stored in a prefix of the form $package/$version/ because some packages do not
/// include the version in the name of their assets. If the assets were not downloaded in a
/// unique-per-version directory then the cache could be misled into thinking it already has
/// downloaded an asset when it's in fact the asset from a previous version.
impl FileCache {
    pub fn new(dir: &Path) -> FileCache {
        FileCache {
            dir: PathBuf::from(dir),
        }
    }

    fn get_download_dir(&self, package_name: &str, version: &Version) -> PathBuf {
        self.dir.join(package_name).join(version.to_string())
    }

    pub fn download(
        &self,
        ui: &Ui,
        package_name: &str,
        version: &Version,
        url: &str,
    ) -> Result<PathBuf> {
        let (_, name) = url
            .rsplit_once('/')
            .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;

        let download_dir = self.get_download_dir(package_name, version);
        fs::create_dir_all(&download_dir)?;

        let archive_path = download_dir.join(OsString::from(name));

        if archive_path.exists() {
            ui.info(&format!(
                "{package_name} {version} has already been downloaded"
            ));
        } else {
            download(ui, url, &archive_path)?;
        }

        Ok(archive_path)
    }
}
