use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

use semver::Version;

use crate::arch_os::ArchOs;

#[derive(Debug, Deserialize)]
pub struct Release {
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
    pub releases: HashMap<String, HashMap<String, Release>>, // TODO: Replace String with ArchOs
}

impl Package {
    pub fn from_file(path: &Path) -> Result<Package> {
        let file = File::open(path)?;
        let package = serde_yaml::from_reader(file)?;
        Ok(package)
    }

    pub fn get_latest_release(&self) -> Option<&Release> {
        let max_entry = self
            .releases
            .iter()
            .max_by_key(|(version, _)| Version::parse(version).unwrap())?;

        let arch_os = ArchOs::current().to_str();
        max_entry.1.get(&arch_os)
    }
}
