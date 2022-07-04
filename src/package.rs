use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::arch_os::ArchOs;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Install {
    pub strip: u32,
    pub files: BTreeMap<String, String>,
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

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub releases: BTreeMap<Version, HashMap<ArchOs, Build>>,

    installs: BTreeMap<Version, HashMap<ArchOs, Install>>,
}

/// Intermediate struct, used to serialize and deserialize. After deserializing it is turned into
/// Package, which has stronger typing
#[derive(Debug, Deserialize, Serialize)]
pub struct InternalPackage {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub releases: Option<BTreeMap<String, BTreeMap<String, Build>>>,
    pub installs: Option<BTreeMap<String, BTreeMap<String, Install>>>,
}

impl InternalPackage {
    fn from_package(package: &Package) -> InternalPackage {
        let mut releases = BTreeMap::<String, BTreeMap<String, Build>>::new();
        for (version, release) in package.releases.iter() {
            let version_str = version.to_string();
            let release = release
                .iter()
                .map(|(arch_os, build)| (arch_os.to_str(), build.clone()))
                .collect();
            releases.insert(version_str, release);
        }

        let mut installs = BTreeMap::<String, BTreeMap<String, Install>>::new();
        for (version, installs_for_arch_os) in package.installs.iter() {
            let version_str = version.to_string();
            let installs_for_arch_os = installs_for_arch_os
                .iter()
                .map(|(arch_os, install)| (arch_os.to_str(), install.clone()))
                .collect();
            installs.insert(version_str, installs_for_arch_os);
        }

        InternalPackage {
            name: package.name.clone(),
            description: package.description.clone(),
            homepage: package.homepage.clone(),
            releases: Some(releases),
            installs: Some(installs),
        }
    }

    fn to_package(&self) -> Result<Package> {
        let mut releases = BTreeMap::<Version, HashMap<ArchOs, Build>>::new();
        if let Some(internal_releases) = &self.releases {
            for (version_str, builds_for_arch_os) in internal_releases.iter() {
                let version = Version::parse(version_str)?;
                let builds_for_arch_os = builds_for_arch_os
                    .iter()
                    .map(|(arch_os, build)| (ArchOs::parse(arch_os).unwrap(), build.clone()))
                    .collect();
                releases.insert(version, builds_for_arch_os);
            }
        }

        let mut installs = BTreeMap::<Version, HashMap<ArchOs, Install>>::new();
        if let Some(internal_installs) = &self.installs {
            for (version_str, installs_for_arch_os) in internal_installs.iter() {
                let version = Version::parse(version_str)?;
                let installs_for_arch_os = installs_for_arch_os
                    .iter()
                    .map(|(arch_os, install)| {
                        (ArchOs::parse(arch_os).unwrap(), Install::expanded(install))
                    })
                    .collect();
                installs.insert(version, installs_for_arch_os);
            }
        }

        Ok(Package {
            name: self.name.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
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

    pub fn from_yaml_str(yaml_str: &str) -> Result<Package> {
        let internal_package: InternalPackage = serde_yaml::from_str(yaml_str)?;
        internal_package.to_package()
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let internal_package = InternalPackage::from_package(self);
        let file = File::create(path)?;
        serde_yaml::to_writer(file, &internal_package)?;
        Ok(())
    }

    /// Returns a clone of the package with the builds for version `version` replaced by
    /// those from `release`
    pub fn replace_release(&self, version: &Version, release: HashMap<ArchOs, Build>) -> Package {
        let mut releases = self.releases.clone();
        releases.insert(version.clone(), release);
        Package {
            name: self.name.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
            releases,
            installs: self.installs.clone(),
        }
    }

    pub fn get_version_matching(&self, requested_version: &VersionReq) -> Option<&Version> {
        self.releases
            .keys()
            .rev()
            .find(|&version| requested_version.matches(version))
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
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
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

        let install = package
            .get_install(&Version::new(1, 2, 0), &ArchOs::current())
            .unwrap();
        assert!(install.files.get("bin/foo-1.2") == Some(&"bin/foo".to_string()));
        assert!(install.files.get("share") == Some(&"share".to_string()));
    }

    #[test]
    fn test_get_version_matching() {
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
              2.0.0:
                any:
                  url: https://example.com
                  sha256: 1234
              1.2.1:
                any:
                  url: https://example.com
                  sha256: 1234
              1.2.0:
                any:
                  url: https://example.com
                  sha256: 1234
            installs: {}
            ",
        )
        .unwrap();

        let req300 = VersionReq::parse("3.0.0").unwrap();
        let req121 = VersionReq::parse("1.2.1").unwrap();
        let req12 = VersionReq::parse("1.2.*").unwrap();
        let req2 = VersionReq::parse(">=2").unwrap();

        let v121 = Version::new(1, 2, 1);
        let v200 = Version::new(2, 0, 0);

        assert!(package.get_version_matching(&req300) == None);
        assert!(package.get_version_matching(&req121) == Some(&v121));
        assert!(package.get_version_matching(&req12) == Some(&v121));
        assert!(package.get_version_matching(&req2) == Some(&v200));
    }
}
