use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub binaries: HashMap<String, String>,
}

impl Release {
    pub fn get_archive_name(&self) -> Result<OsString> {
        let (_, name) = self
            .url
            .rsplit_once('/')
            .ok_or_else(|| anyhow!("Can't find archive name in URL {}", self.url))?;

        Ok(OsString::from(name))
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub releases: Vec<Release>,
}

impl Package {
    pub fn from_file(path: &Path) -> Result<Package> {
        let file = File::open(path)?;
        let package = serde_yaml::from_reader(file)?;
        Ok(package)
    }
}
