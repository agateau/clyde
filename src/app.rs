use std::boxed::Box;
use std::path::{Path, PathBuf};

use crate::file_cache::FileCache;
use crate::store::{GitStore, Store};

pub struct App {
    pub download_cache: FileCache,
    pub bin_dir: PathBuf,
    pub pkg_base_dir: PathBuf,
    pub store: Box<dyn Store>,
}

impl App {
    pub fn new(prefix: &Path) -> App {
        let store_dir = prefix.join("store");
        App {
            download_cache: FileCache::new(Path::new("/tmp")),
            bin_dir: prefix.join("bin"),
            pkg_base_dir: prefix.join("pkg"),
            store: Box::new(GitStore::new(&store_dir)),
        }
    }
}
