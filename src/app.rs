use std::path::{Path, PathBuf};

use crate::file_cache::FileCache;

pub struct App {
    pub download_cache: FileCache,
    pub bin_dir: PathBuf
}

impl App {
    pub fn new(prefix: &Path) -> App {
        App {
            download_cache: FileCache::new(Path::new("/tmp")),
            bin_dir: prefix.join("lib").join("pinky")
        }
    }
}
