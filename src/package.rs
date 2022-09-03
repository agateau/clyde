// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::Path;

use anyhow::Result;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::arch_os::{ArchOs, ANY};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
}

pub type Release = HashMap<ArchOs, Asset>;

fn is_zero(x: &u32) -> bool {
    *x == 0
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Install {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
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
                    if dst.is_empty() {
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
    pub repository: String,
    pub releases: BTreeMap<Version, Release>,

    pub installs: BTreeMap<Version, HashMap<ArchOs, Install>>,
}

/// Intermediate struct, used to serialize and deserialize. After deserializing it is turned into
/// Package, which has stronger typing
#[derive(Debug, Deserialize, Serialize)]
struct InternalPackage {
    pub name: String,
    pub description: String,
    pub homepage: String,
    #[serde(default)]
    pub repository: String,
    pub releases: Option<BTreeMap<String, BTreeMap<String, Asset>>>,
    pub installs: Option<BTreeMap<String, BTreeMap<String, Install>>>,
}

impl InternalPackage {
    fn from_package(package: &Package) -> InternalPackage {
        let mut releases = BTreeMap::<String, BTreeMap<String, Asset>>::new();
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
            repository: package.repository.clone(),
            releases: Some(releases),
            installs: Some(installs),
        }
    }

    fn to_package(&self) -> Result<Package> {
        let mut releases = BTreeMap::<Version, Release>::new();
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
            repository: self.repository.clone(),
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
    pub fn replace_release(&self, version: &Version, release: Release) -> Package {
        let mut releases = self.releases.clone();
        releases.insert(version.clone(), release);
        Package {
            name: self.name.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
            repository: self.repository.clone(),
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

    pub fn get_asset(&self, version: &Version, arch_os: &ArchOs) -> Option<&Asset> {
        let release = self.releases.get(version)?;
        release.get(arch_os)
    }

    /// Return files definition for wanted_version
    /// Uses the highest version which is less or equal to wanted_version
    pub fn get_install(&self, wanted_version: &Version, arch_os: &ArchOs) -> Option<&Install> {
        let install = self.get_install_internal(wanted_version, arch_os);
        if install.is_some() {
            return install;
        }
        if arch_os.arch != ANY {
            let arch_os = ArchOs::new(ANY, &arch_os.os);
            let install = self.get_install_internal(wanted_version, &arch_os);
            if install.is_some() {
                return install;
            }
        }
        if arch_os.os != ANY {
            // Probably less useful than the previous check, but you never know
            let arch_os = ArchOs::new(&arch_os.arch, ANY);
            let install = self.get_install_internal(wanted_version, &arch_os);
            if install.is_some() {
                return install;
            }
        }
        self.get_install_internal(wanted_version, &ArchOs::new(ANY, ANY))
    }

    fn get_install_internal(&self, wanted_version: &Version, arch_os: &ArchOs) -> Option<&Install> {
        let entry = self
            .installs
            .iter()
            .rev()
            .find(|(version, _)| *version <= wanted_version)?;
        let installs_for_arch_os = entry.1;
        installs_for_arch_os.get(arch_os)
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
        assert_eq!(
            install.files.get("bin/foo-1.2"),
            Some(&"bin/foo".to_string())
        );
        assert_eq!(install.files.get("share"), Some(&"share".to_string()));
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

        assert_eq!(package.get_version_matching(&req300), None);
        assert_eq!(package.get_version_matching(&req121), Some(&v121));
        assert_eq!(package.get_version_matching(&req12), Some(&v121));
        assert_eq!(package.get_version_matching(&req2), Some(&v200));
    }

    #[test]
    fn get_install_should_use_the_any_arch_specific_os_install() {
        // GIVEN a package with any and any-macos installs
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases: {}
            installs:
              1.0.0:
                any:
                  strip: 1
                  files:
                    foo:
                any-macos:
                  strip: 3
                  files:
                    foo:
            ",
        )
        .unwrap();

        // WHEN installing on macos
        let install = package
            .get_install(&Version::new(1, 0, 0), &ArchOs::new("x86_64", "macos"))
            .unwrap();

        // THEN the any-macos install is used
        assert_eq!(install.strip, 3);
    }

    #[test]
    fn strip_should_default_to_0_if_not_set() {
        // GIVEN a package with no value for `strip`
        // WHEN parsing it
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases: {}
            installs:
              1.0.0:
                any:
                  files:
                    foo:
            ",
        );

        // THEN it succeeds
        let package = package.unwrap();

        // AND strip is 0
        let install = package
            .get_install(&Version::new(1, 0, 0), &ArchOs::new(ANY, ANY))
            .unwrap();
        assert_eq!(install.strip, 0);
    }
}
