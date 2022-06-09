use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::package::Package;

pub trait Store {
    fn update(&self) -> Result<()>;
    fn get_package(&self, name: &str) -> Result<Package>;
}

pub struct GitStore {
    dir: PathBuf,
}

impl GitStore {
    pub fn new(dir: &Path) -> GitStore {
        GitStore {
            dir: dir.to_path_buf(),
        }
    }

    fn find_package_path(&self, name: &str) -> Option<PathBuf> {
        let direct_path = PathBuf::from(name);
        if direct_path.is_file() {
            return Some(direct_path);
        }
        let store_path = self.dir.join(name.to_owned() + ".yaml");
        if store_path.is_file() {
            return Some(store_path);
        }
        None
    }
}

impl Store for GitStore {
    fn update(&self) -> Result<()> {
        Ok(())
    }

    fn get_package(&self, name: &str) -> Result<Package> {
        let path = self
            .find_package_path(&name)
            .ok_or(anyhow!("No such package: {}", name))?;
        Package::from_file(&path)
    }
}
