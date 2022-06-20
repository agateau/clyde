use std::path::{Path, PathBuf};
use std::process::Command;

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
        let mut cmd = Command::new("git");
        cmd.arg("-C");
        cmd.arg(self.dir.as_os_str());
        cmd.arg("pull");
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Failed to update"));
        }
        Ok(())
    }

    fn get_package(&self, name: &str) -> Result<Package> {
        let path = self
            .find_package_path(name)
            .ok_or_else(|| anyhow!("No such package: {}", name))?;
        Package::from_file(&path)
    }
}
