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

#[cfg(test)]
mod tests {
    use yare::parameterized;

    use super::*;

    use crate::test_file_utils::{get_fixture_path, list_tree, pathbufset_from_strings};

    #[parameterized(
        tar_gz = { "test_archive.tar.gz" },
        tar_bz2 = { "test_archive.tar.bz2" },
        tar_xz = { "test_archive.tar.xz" },
    )]
    fn test_unpack(filename: &str) {
        // GIVEN a compressed tar archive
        let path = get_fixture_path(filename);

        let dir = assert_fs::TempDir::new().unwrap();

        // AND a TarUnpacker on it
        let unpacker = TarUnpacker::new(&path);

        // WHEN unpack() is called
        let ret = unpacker.unpack(&dir, 0).unwrap();
        assert!(ret.is_none());

        // THEN the archive content is copied in dir
        let actual_files = list_tree(&dir).unwrap();
        let expected_files = pathbufset_from_strings(&[
            "hello/bin/hello",
            "hello/bin/hello-symlink",
            "hello/README.md",
        ]);
        assert_eq!(actual_files, expected_files);

        #[cfg(unix)]
        {
            // AND hello/bin/hello is executable
            use crate::test_file_utils::is_file_executable;
            let exe_path = dir.join("hello/bin/hello").canonicalize().unwrap();
            assert!(is_file_executable(&exe_path));

            // AND hello/bin/hello-symlink points to hello/bin/hello
            let symlink_path = dir.join("hello/bin/hello-symlink");
            assert!(symlink_path.exists());
            assert!(symlink_path.is_symlink());
            let target_path = symlink_path.canonicalize().unwrap();
            assert_eq!(target_path, exe_path);
        }
    }

    #[test]
    fn test_strip_components() {
        // GIVEN a compressed tar archive
        let path = get_fixture_path("test_archive.tar.gz");

        let dir = assert_fs::TempDir::new().unwrap();

        // AND a TarUnpacker on it
        let unpacker = TarUnpacker::new(&path);

        // WHEN unpack() is called with strip = 1
        let ret = unpacker.unpack(&dir, 1).unwrap();
        assert!(ret.is_none());

        // THEN the archive content is copied in dir, but the top-level directory is not there
        let actual_files = list_tree(&dir).unwrap();
        let expected_files =
            pathbufset_from_strings(&["bin/hello", "bin/hello-symlink", "README.md"]);
        assert_eq!(actual_files, expected_files);
    }
}
