use std::boxed::Box;
use std::path::{Path, PathBuf};

use crate::file_cache::FileCache;
use crate::store::{GitStore, Store};

pub struct App {
    pub download_cache: FileCache,
    pub install_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub store: Box<dyn Store>,
}

impl App {
    pub fn new(prefix: &Path) -> App {
        let store_dir = prefix.join("store");
        App {
            download_cache: FileCache::new(Path::new("/tmp")),
            install_dir: prefix.join("inst"),
            tmp_dir: prefix.join("tmp"),
            store: Box::new(GitStore::new(&store_dir)),
        }
    }
}
