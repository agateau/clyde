// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::unpacker::Unpacker;

pub struct ZipUnpacker {
    archive_path: PathBuf,
}

impl ZipUnpacker {
    pub fn new(archive: &Path) -> ZipUnpacker {
        ZipUnpacker {
            archive_path: archive.to_path_buf(),
        }
    }

    pub fn supports(name: &str) -> bool {
        name.ends_with(".zip")
    }
}

fn apply_strip(path: &Path, strip: u32) -> Option<PathBuf> {
    if strip == 0 {
        return Some(path.to_owned());
    }
    let prefix = path.iter().next()?;

    let path = path.strip_prefix(prefix).ok()?;
    if path == Path::new("") {
        return None;
    }
    apply_strip(path, strip - 1)
}

impl Unpacker for ZipUnpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<()> {
        let archive_file = fs::File::open(&self.archive_path)
            .with_context(|| format!("Failed to open {:?}", self.archive_path))?;

        let mut archive = ZipArchive::new(archive_file)
            .with_context(|| format!("Failed to read {:?}", self.archive_path))?;

        for idx in 0..archive.len() {
            let mut file = archive.by_index(idx)?;
            let dst_sub_path = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let dst_sub_path = match apply_strip(&dst_sub_path, strip) {
                Some(x) => x,
                None => continue,
            };

            let dst_path = dst_dir.join(&dst_sub_path);

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&dst_path)
                    .with_context(|| format!("Failed to create directory {dst_path:?}"))?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(&parent)
                            .with_context(|| format!("Failed to create directory {parent:?}"))?;
                    }
                }
                let mut dst_file = fs::File::create(&dst_path)
                    .with_context(|| format!("Failed to create file {dst_path:?}"))?;
                io::copy(&mut file, &mut dst_file).with_context(|| {
                    format!("Failed to write {:?} to {dst_path:?}", file.name())
                })?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&dst_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_file_utils::{create_test_zip_file, list_tree, pathbufset_from_strings};

    #[test]
    fn apply_strip_should_strip_1() {
        assert_eq!(
            apply_strip(Path::new("foo/bar"), 1),
            Some(PathBuf::from("bar"))
        );
    }

    #[test]
    fn apply_strip_should_return_none_when_stripping_too_much() {
        assert_eq!(apply_strip(Path::new("foo/bar"), 2), None);
    }

    #[test]
    fn unpack_should_unpack_in_the_right_dir() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN the test zip file
        let zip_path = dir.join("test.zip");
        create_test_zip_file(&zip_path);

        // AND an unpacker on this zip file
        let unpacker = ZipUnpacker::new(&zip_path);

        // WHEN unpack() is called in a subdir of `dir`
        let dst_dir = dir.join("sub");
        unpacker.unpack(&dst_dir, 0).unwrap();

        // THEN the zip file is unpacked there
        assert_eq!(
            list_tree(&dst_dir).unwrap(),
            pathbufset_from_strings(&["hello/bin/hello", "hello/README.md"])
        );
    }

    #[test]
    fn unpack_should_honor_strip() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a zip file with the following content:
        // hello/
        // hello/bin/
        // hello/bin/hello
        // hello/README.md
        let zip_path = dir.join("test.zip");
        create_test_zip_file(&zip_path);

        // AND an unpacker on this zip file
        let unpacker = ZipUnpacker::new(&zip_path);

        // WHEN unpack() is called in a subdir of `dir` with a strip of 1
        let dst_dir = dir.join("sub");
        unpacker.unpack(&dst_dir, 1).unwrap();

        // THEN the zip file is unpacked as expected
        assert_eq!(
            list_tree(&dst_dir).unwrap(),
            pathbufset_from_strings(&["bin/hello", "README.md"])
        );
    }

    #[test]
    fn unpack_should_ignore_files_outside_the_strip() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a zip file with the following content:
        // hello/
        // hello/bin/
        // hello/bin/hello
        // hello/README.md
        let zip_path = dir.join("test.zip");
        create_test_zip_file(&zip_path);

        // AND an unpacker on this zip file
        let unpacker = ZipUnpacker::new(&zip_path);

        // WHEN unpack() is called in a subdir of `dir` with a strip of 2
        let dst_dir = dir.join("sub");
        unpacker.unpack(&dst_dir, 2).unwrap();

        // THEN the zip file is unpacked as expected
        assert_eq!(
            list_tree(&dst_dir).unwrap(),
            pathbufset_from_strings(&["hello"])
        );
    }
}
