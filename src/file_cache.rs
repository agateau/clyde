use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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
}
