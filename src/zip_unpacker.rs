use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
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
}

fn apply_strip(path: &Path, strip: u32) -> Option<PathBuf> {
    if strip == 0 {
        return Some(path.to_owned());
    }
    let prefix = path.iter().next()?;

    let path = path.strip_prefix(prefix).ok()?;
    Some(path.to_owned())
}

impl Unpacker for ZipUnpacker {
    fn unpack(&self, dst_dir: &Path, strip: u32) -> Result<()> {
        let archive_file = fs::File::open(&self.archive_path)
            .map_err(|x| anyhow!("Error with {:?}: {}", self.archive_path, x))?;

        let mut archive = ZipArchive::new(archive_file)
            .map_err(|x| anyhow!("Failed to open {:?}: {}", self.archive_path, x))?;

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
                println!("File {} extracted to \"{}\"", idx, dst_path.display());
                fs::create_dir_all(&dst_path)?;
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    idx,
                    dst_path.display(),
                    file.size()
                );
                if let Some(parent) = dst_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(&parent)?;
                    }
                }
                let mut dst_file = fs::File::create(&dst_path)?;
                io::copy(&mut file, &mut dst_file)?;
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

    use std::include_bytes;

    use crate::test_file_utils::{list_tree, pathbufset_from_strings};

    // Zip content:
    // hello/
    // hello/bin/
    // hello/bin/hello
    // hello/README.md
    const ZIP_BYTES: &[u8; 626] = include_bytes!("zip_unpacker_test_archive.zip");

    #[test]
    fn unpack_should_unpack_in_the_right_dir() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN the test zip file
        let zip_path = dir.join("test.zip");
        fs::write(&zip_path, ZIP_BYTES).unwrap();

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
        fs::write(&zip_path, ZIP_BYTES).unwrap();

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
}
