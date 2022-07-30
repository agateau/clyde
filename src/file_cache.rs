// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::download::download;
use crate::ui::Ui;

pub struct FileCache {
    dir: PathBuf,
}

impl FileCache {
    pub fn new(dir: &Path) -> FileCache {
        FileCache {
            dir: PathBuf::from(dir),
        }
    }

    pub fn get_path(&self, name: &OsStr) -> PathBuf {
        self.dir.join(name)
    }

    pub fn download(&self, ui: &Ui, url: &str) -> Result<PathBuf> {
        let (_, name) = url
            .rsplit_once('/')
            .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;

        let archive_path = self.get_path(&OsString::from(name));

        if !archive_path.exists() {
            download(ui, url, &archive_path)?;
        }

        Ok(archive_path)
    }
}
