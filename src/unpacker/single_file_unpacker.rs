// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use archiver_rs::Compressed;

use crate::file_utils;
use crate::unpacker::{tar_unpacker::TarUnpacker, Unpacker};

/// An "unpacker" for archives which are a single compressed file (for example foo.gz)
pub struct SingleFileUnpacker {
    archive_path: PathBuf,
}

impl SingleFileUnpacker {
    pub fn new(archive: &Path) -> SingleFileUnpacker {
        SingleFileUnpacker {
            archive_path: archive.to_path_buf(),
        }
    }

    pub fn supports(name: &str) -> bool {
        if TarUnpacker::supports(name) {
            return false;
        }
        name.ends_with(".gz") || name.ends_with(".bz2") || name.ends_with(".xz2")
    }
}

impl Unpacker for SingleFileUnpacker {
    fn unpack(&self, dst_dir: &Path, _strip: u32) -> Result<Option<String>> {
        let extension = self
            .archive_path
            .extension()
            .ok_or_else(|| anyhow!("Can't get extension for {:?}", self.archive_path))?
            .to_str()
            .unwrap();
        let dst_path = dst_dir.join(self.archive_path.file_stem().unwrap());

        let mut compressed: Box<dyn Compressed> = match extension {
            "gz" => Box::new(archiver_rs::Gzip::open(&self.archive_path)?),
            "bz2" => Box::new(archiver_rs::Bzip2::open(&self.archive_path)?),
            "xz" => Box::new(archiver_rs::Xz::open(&self.archive_path)?),
            _ => {
                return Err(anyhow!("Don't know how to unpack {:?}", self.archive_path));
            }
        };
        compressed.decompress(&dst_path)?;

        #[cfg(unix)]
        file_utils::set_file_executable(&dst_path)?;

        let name = file_utils::get_file_name(&dst_path)?;
        Ok(Some(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_file_utils::get_fixture_path;

    #[test]
    fn supports_should_not_accept_tar_compressed_files() {
        assert!(!SingleFileUnpacker::supports("foo.tar.gz"));
        assert!(!SingleFileUnpacker::supports("foo.tar.bz2"));
        assert!(!SingleFileUnpacker::supports("foo.tar.xz"));
    }

    fn check_unpack_should_copy_file(compressed_exe_path: &Path) {
        // GIVEN a compressed file
        let exe_file_name = compressed_exe_path.file_stem().unwrap();

        let dir = assert_fs::TempDir::new().unwrap();

        // AND a SingleFileUnpacker on it
        let unpacker = SingleFileUnpacker {
            archive_path: compressed_exe_path.to_path_buf(),
        };

        // WHEN unpack() is called
        unpacker.unpack(&dir, 0).unwrap();

        // THEN the executable is copied there
        let dst_path = dir.join(exe_file_name);
        assert!(dst_path.exists());

        // AND the executable has the required permission
        #[cfg(unix)]
        {
            use crate::test_file_utils::is_file_executable;
            assert!(is_file_executable(&dst_path));
        }
    }

    #[test]
    fn unpack_should_copy_file() {
        check_unpack_should_copy_file(&get_fixture_path("test_exe.gz"));
        check_unpack_should_copy_file(&get_fixture_path("test_exe.bz2"));
        check_unpack_should_copy_file(&get_fixture_path("test_exe.xz"));
    }
}
