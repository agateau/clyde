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

use crate::unpacker::unpacker_utils::check_path_is_safe;
// use crate::unpacker::unpacker_utils::validate_inside_dst;
use crate::vendored::archiver_rs;

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
            let path = entry.path()?;

            let path = match apply_strip(&path, strip) {
                Some(x) => x,
                None => continue,
            };
            check_path_is_safe(&path)?;

            let entry_type = entry.header().entry_type();
            if entry_type != tar::EntryType::Directory {
                if let Some(link_path) = entry.link_name()? {
                    let target = path.parent().unwrap_or(Path::new(".")).join(link_path);
                    check_path_is_safe(&target)?;
                }
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

    #[test]
    fn does_not_extract_in_dir_parent() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a compressed tar archive that contains a ../evil file
        let path = get_fixture_path("evil.tar.gz");

        // AND a TarUnpacker on it
        let unpacker = TarUnpacker::new(&path);

        // AND a sub_dir inside the test dir
        let sub_dir = dir.join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();

        // WHEN unpack() is called in sub_dir
        let result = unpacker.unpack(&sub_dir, 0);

        // THEN the evil file is not created
        let evil_path = dir.join("evil");
        assert!(!evil_path.exists());

        // AND the result is a failure
        assert!(result.is_err());
    }

    #[test]
    fn does_not_extract_bad_symlinks() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a compressed tar archive that contains a ../evil file
        let path = get_fixture_path("evil_link.tar.gz");

        // AND a TarUnpacker on it
        let unpacker = TarUnpacker::new(&path);

        // AND a sub_dir inside the test dir
        let sub_dir = dir.join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();

        fs::create_dir(dir.join("dir")).unwrap();

        // WHEN unpack() is called in sub_dir
        let result = unpacker.unpack(&sub_dir, 0);

        // THEN the evil file is not created
        let evil_path = dir.join("dir").join("payload");
        assert!(!evil_path.exists());

        // AND the result is a failure
        assert!(result.is_err());
    }

    fn assert_same_files(path1: &Path, path2: &Path) {
        let path1 = path1.canonicalize().unwrap();
        let path2 = path2.canonicalize().unwrap();
        assert_eq!(path1, path2);
    }

    // This test does not pass on Windows. `assert!(symlink.exists())` fails.
    // Disable it there for now.
    #[cfg(not(windows))]
    #[test]
    fn extracts_good_symlinks() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a compressed tar archive that contains a symlink
        // that goes one level up to another file in the archive
        let path = get_fixture_path("ok_link.tar.gz");

        // AND a TarUnpacker on it
        let unpacker = TarUnpacker::new(&path);

        // WHEN unpack() is called in dir
        let result = unpacker.unpack(&dir, 0);

        // THEN the archive is unpacked
        let symlink = dir.join("dir1").join("symlink");
        let real_file = dir.join("dir2").join("real_file");
        let dir_symlink = dir.join("dir_symlink");

        // AND "dir1/symlink" points to "dir2/real_file"
        assert!(symlink.exists());
        assert!(real_file.exists());

        // AND "dir_symlink" points to "dir2"
        assert!(dir_symlink.exists());
        assert_same_files(&symlink, &real_file);
        assert_same_files(&dir_symlink, &dir.join("dir2"));

        // AND the result is OK
        assert!(result.is_ok());
    }
}
