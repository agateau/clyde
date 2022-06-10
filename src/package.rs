use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

use semver::Version;

use crate::arch_os::ArchOs;

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub releases: HashMap<Version, HashMap<ArchOs, Release>>,
}

/// Internal class, used to deserialize: this is then turned into Package, which has stronger
/// typing
#[derive(Debug, Deserialize)]
struct InternalPackage {
    pub name: String,
    pub description: String,
    pub releases: HashMap<String, HashMap<String, Release>>,
}

impl InternalPackage {
    fn to_package(&self) -> Result<Package> {
        let mut releases = HashMap::<Version, HashMap<ArchOs, Release>>::new();
        for (version_str, internal_release_map) in self.releases.iter() {
            let version = Version::parse(version_str)?;
            let release_map = internal_release_map
                .iter()
                .map(|(arch_os, release)| (ArchOs::parse(arch_os).unwrap(), release.clone()))
                .collect();
            releases.insert(version, release_map);
        }
        Ok(Package {
            name: self.name.clone(),
            description: self.description.clone(),
            releases,
        })
    }
}

impl Package {
    pub fn from_file(path: &Path) -> Result<Package> {
        let file = File::open(path)?;
        let internal_package: InternalPackage = serde_yaml::from_reader(file)?;
        internal_package.to_package()
    }

    pub fn get_latest_release(&self) -> Option<&Release> {
        let max_entry = self
            .releases
            .iter()
            .max_by_key(|(version, _)| version.clone())?;

        let arch_os = ArchOs::current();
        max_entry.1.get(&arch_os)
    }
}
