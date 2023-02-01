// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::unpacker::Unpacker;

pub struct TarUnpacker {
    pub archive: PathBuf,
}

impl TarUnpacker {
    pub fn new(archive: &Path) -> TarUnpacker {
        TarUnpacker {
            archive: archive.to_path_buf(),
        }
    }

    pub fn supports(name: &str) -> bool {
        name.ends_with(".tar.gz")
            || name.ends_with(".tar.bz2")
            || name.ends_with(".tar.xz")
            || name.ends_with(".tgz")
            || name.ends_with(".tbz2")
    }
}

impl Unpacker for TarUnpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<Option<String>> {
        fs::create_dir_all(dst_dir)?;

        let mut cmd = Command::new("tar");
        cmd.arg("-C");
        cmd.arg(dst_dir);
        if strip > 0 {
            cmd.arg(format!("--strip-components={strip}"));
        }
        cmd.arg("-xf");
        cmd.arg(self.archive.canonicalize()?);
        let status = match cmd.status() {
            Ok(x) => x,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Err(anyhow!(
                        "Failed to unpack {}: tar is not installed",
                        self.archive.display()
                    ));
                } else {
                    return Err(err.into());
                }
            }
        };

        if !status.success() {
            return Err(anyhow!("Error unpacking {}", self.archive.display()));
        }

        Ok(None)
    }
}
