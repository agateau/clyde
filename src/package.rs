use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

use semver::Version;

use crate::arch_os::ArchOs;

#[derive(Debug, Deserialize, Clone)]
pub struct Build {
    pub url: String,
    pub sha256: String,
}

impl Build {
    pub fn get_archive_name(&self) -> Result<OsString> {
        let (_, name) = self
            .url
            .rsplit_once('/')
            .ok_or_else(|| anyhow!("Can't find archive name in URL {}", self.url))?;

        Ok(OsString::from(name))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Install {
    pub strip: u32,
    pub files: HashMap<String, String>,
}

impl Install {
    pub fn expanded(other: &Install) -> Install {
        let files = other
            .files
            .iter()
            .map(|(src, dst)| {
                (
                    src.clone(),
                    // The == "~" is there to workaround https://github.com/dtolnay/serde-yaml/issues/87
                    if dst.is_empty() || dst == "~" {
                        src.clone()
                    } else {
                        dst.clone()
                    },
                )
            })
            .collect();
        Install {
            strip: other.strip,
            files,
        }
    }
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub releases: BTreeMap<Version, HashMap<ArchOs, Build>>,

    installs: BTreeMap<Version, HashMap<ArchOs, Install>>,
}

/// Internal class, used to deserialize: this is then turned into Package, which has stronger
/// typing
#[derive(Debug, Deserialize)]
struct InternalPackage {
    pub name: String,
    pub description: String,
    pub releases: HashMap<String, HashMap<String, Build>>,
    pub installs: HashMap<String, HashMap<String, Install>>,
}

impl InternalPackage {
    fn to_package(&self) -> Result<Package> {
        let mut releases = BTreeMap::<Version, HashMap<ArchOs, Build>>::new();
        for (version_str, builds_for_arch_os) in self.releases.iter() {
            let version = Version::parse(version_str)?;
            let builds_for_arch_os = builds_for_arch_os
                .iter()
                .map(|(arch_os, build)| (ArchOs::parse(arch_os).unwrap(), build.clone()))
                .collect();
            releases.insert(version, builds_for_arch_os);
        }

        let mut installs = BTreeMap::<Version, HashMap<ArchOs, Install>>::new();
        for (version_str, installs_for_arch_os) in self.installs.iter() {
            let version = Version::parse(version_str)?;
            let installs_for_arch_os = installs_for_arch_os
                .iter()
                .map(|(arch_os, install)| {
                    (ArchOs::parse(arch_os).unwrap(), Install::expanded(install))
                })
                .collect();
            installs.insert(version, installs_for_arch_os);
        }

        Ok(Package {
            name: self.name.clone(),
            description: self.description.clone(),
            releases,
            installs,
        })
    }
}

impl Package {
    pub fn from_file(path: &Path) -> Result<Package> {
        let file = File::open(path)?;
        let internal_package: InternalPackage = serde_yaml::from_reader(file)?;
        internal_package.to_package()
    }

    pub fn get_latest_version(&self) -> Option<&Version> {
        let entry = self.releases.iter().last()?;
        Some(entry.0)
    }

    pub fn get_build(&self, version: &Version, arch_os: &ArchOs) -> Option<&Build> {
        let release = self.releases.get(version)?;
        release.get(arch_os)
    }

    /// Return files definition for wanted_version
    /// Uses the highest version which is less or equal to wanted_version
    pub fn get_install(&self, wanted_version: &Version, arch_os: &ArchOs) -> Option<&Install> {
        let entry = self
            .installs
            .iter()
            .rev()
            .find(|(version, _)| *version <= wanted_version)?;
        let installs_for_arch_os = entry.1;
        if let Some(install) = installs_for_arch_os.get(arch_os) {
            return Some(install);
        }
        installs_for_arch_os.get(&ArchOs::new("any", "any"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_package() {
        let internal_package: InternalPackage = serde_yaml::from_str(
            "
            name: test
            description: desc
            releases: {}
            installs:
              1.2.0:
                any:
                  strip: 1
                  files:
                    bin/foo-1.2: bin/foo
                    share:
            ",
        )
        .unwrap();

        let package = internal_package.to_package().unwrap();

        let install = package
            .get_install(&Version::new(1, 2, 0), &ArchOs::current())
            .unwrap();
        assert!(install.files.get("bin/foo-1.2") == Some(&"bin/foo".to_string()));
        assert!(install.files.get("share") == Some(&"share".to_string()));
    }
}
