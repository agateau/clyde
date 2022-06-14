use std::boxed::Box;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use crate::file_cache::FileCache;
use crate::store::{GitStore, Store};

pub struct App {
    pub download_cache: FileCache,
    pub install_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub store: Box<dyn Store>,
}

impl App {
    pub fn find_prefix() -> Result<PathBuf> {
        if let Some(prefix) = env::var_os("CLYDE_PREFIX") {
            return Ok(Path::new(&prefix).to_path_buf());
        }

        if let Some(prefix_path) = ProjectDirs::from("", "", "clyde") {
            return Ok(prefix_path.cache_dir().to_path_buf());
        }

        Err(anyhow!("Could not find a path for Clyde prefix"))
    }

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
