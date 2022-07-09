use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use goblin::{self, Hint};

use crate::unpacker::Unpacker;

/// An "unpacker" for archives which are actually directly an executable
pub struct ExeUnpacker {
    archive_path: PathBuf,
}

impl ExeUnpacker {
    pub fn new(archive: &Path) -> ExeUnpacker {
        ExeUnpacker {
            archive_path: archive.to_path_buf(),
        }
    }

    pub fn supports(archive_path: &Path) -> bool {
        let mut file = match File::open(archive_path) {
            Ok(x) => x,
            Err(_) => {
                return false;
            }
        };
        let hint = match goblin::peek(&mut file) {
            Ok(x) => x,
            Err(_) => {
                return false;
            }
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        return matches!(hint, Hint::Elf(_));

        #[cfg(target_os = "macos")]
        return matches!(hint, Hint::Mach(_) | Hint::MachFat(_));

        #[cfg(windows)]
        matches!(hint, Hint::PE)
    }
}

impl Unpacker for ExeUnpacker {
    fn unpack(&self, dst_dir: &Path, _strip: u32) -> Result<()> {
        let exe_file_name = self.archive_path.file_name().unwrap();

        let dst_path = dst_dir.join(exe_file_name);
        fs::create_dir_all(&dst_path.parent().unwrap())?;

        let mut src_file = File::open(&self.archive_path)
            .with_context(|| format!("Error with {:?}", self.archive_path))?;

        let mut dst_file =
            File::create(&dst_path).with_context(|| format!("Can't create {:?}", dst_path))?;

        io::copy(&mut src_file, &mut dst_file)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::metadata(&dst_path).unwrap().permissions();
            let mode = permissions.mode() | 0o111;
            fs::set_permissions(&dst_path, fs::Permissions::from_mode(mode))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_file_utils::create_test_zip_file;

    const EXECUTABLE_NAME: &str = if cfg!(unix) {
        "/bin/ls"
    } else if cfg!(windows) {
        "c:\\windows\\explorer.exe"
    } else {
        "not-going-to-work"
    };

    fn get_test_executable_path() -> PathBuf {
        let path = PathBuf::from(EXECUTABLE_NAME);
        assert!(path.exists());
        path
    }

    #[test]
    fn supports_should_accept_executable() {
        let exe_path = get_test_executable_path();
        assert!(ExeUnpacker::supports(&exe_path));
    }

    #[test]
    fn supports_should_not_accept_zip_files() {
        let dir = assert_fs::TempDir::new().unwrap();
        let zip_path = dir.join("test.zip");
        create_test_zip_file(&zip_path);

        assert!(!ExeUnpacker::supports(&zip_path));
    }

    #[test]
    fn unpack_should_copy_file() {
        // GIVEN a copy of EXECUTABLE_NAME
        let src_exe_path = get_test_executable_path();
        let exe_file_name = src_exe_path.file_name().unwrap();

        let dir = assert_fs::TempDir::new().unwrap();
        let dst_exe_path = dir.join(&exe_file_name);
        io::copy(
            &mut File::open(&src_exe_path).unwrap(),
            &mut File::create(&dst_exe_path).unwrap(),
        )
        .unwrap();

        // AND an ExeUnpacker on it
        let unpacker = ExeUnpacker {
            archive_path: dst_exe_path,
        };

        // WHEN unpack() is called in a subdir of `dir`
        let dst_dir = dir.join("sub");
        unpacker.unpack(&dst_dir, 0).unwrap();

        // THEN the executable is copied there
        let dst_path = dst_dir.join(exe_file_name);
        assert!(dst_path.exists());

        // AND the executable has the required permission
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::metadata(&dst_path).unwrap().permissions();
            assert_eq!(permissions.mode() & 0o111_u32, 0o111_u32);
        }
    }
}
