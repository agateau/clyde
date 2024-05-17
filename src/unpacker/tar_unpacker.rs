// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tar::Archive;

use crate::unpacker::unpacker_utils::apply_strip;
use crate::unpacker::Unpacker;

pub struct TarUnpacker {
    pub archive_path: PathBuf,
}

impl TarUnpacker {
    pub fn new(archive_path: &Path) -> TarUnpacker {
        TarUnpacker {
            archive_path: archive_path.to_path_buf(),
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
        let extension = self
            .archive_path
            .extension()
            .ok_or_else(|| anyhow!("Can't get extension for {:?}", self.archive_path))?
            .to_str()
            .unwrap();

        let compressed_reader: Box<dyn io::Read> = match extension {
            "gz" | "tgz" => Box::new(archiver_rs::Gzip::open(&self.archive_path)?),
            "bz2" | "tbz2" => Box::new(archiver_rs::Bzip2::open(&self.archive_path)?),
            "xz" => Box::new(archiver_rs::Xz::open(&self.archive_path)?),
            _ => {
                return Err(anyhow!("Don't know how to unpack {:?}", self.archive_path));
            }
        };

        let mut tar_reader = Archive::new(compressed_reader);

        // Code below is adapted from tar::Archive::unpack()

        // Canonicalizing the dst_dir directory will prepend the path with '\\?\'
        // on windows which will allow windows APIs to treat the path as an
        // extended-length path with a 32,767 character limit. Otherwise all
        // unpacked paths over 260 characters will fail on creation with a
        // NotFound exception.
        let dst_dir = &dst_dir.canonicalize().unwrap_or(dst_dir.to_path_buf());

        fs::create_dir_all(dst_dir)?;

        for entry in tar_reader.entries()? {
            let mut entry = entry?;
            let path = match apply_strip(&entry.path()?, strip) {
                Some(x) => x,
                None => continue,
            };
            if entry.header().entry_type() != tar::EntryType::Directory {
                let entry_path = dst_dir.join(path);
                let entry_dir = entry_path.parent().unwrap();
                fs::create_dir_all(entry_dir)?;
                entry.unpack(entry_path)?;
            }
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
        tgz = { "test_archive.tgz" },
        tbz2 = { "test_archive.tbz2" },
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
