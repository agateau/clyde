use std::boxed::Box;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use crate::db::Database;
use crate::file_cache::FileCache;
use crate::store::{GitStore, Store};

pub struct App {
    pub download_cache: FileCache,
    pub prefix: PathBuf,
    pub install_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub store_dir: PathBuf,
    pub store: Box<dyn Store>,

    db_path: PathBuf,
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
        let store = GitStore::new(&store_dir);

        let db_path = prefix.join("clyde.sqlite");

        App {
            download_cache: FileCache::new(Path::new("/tmp")),
            prefix: prefix.to_path_buf(),
            install_dir: prefix.join("inst"),
            tmp_dir: prefix.join("tmp"),
            store_dir,
            store: Box::new(store),
            db_path,
        }
    }

    pub fn get_database(&self) -> Result<Database> {
        Database::new_from_path(&self.db_path)
    }
}
