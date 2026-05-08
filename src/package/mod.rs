// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod fetcher_config;
mod internal_package;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeDelta, Utc};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

pub use fetcher_config::FetcherConfig;

use crate::arch_os::{Arch, ArchOs, Os};

use internal_package::InternalPackage;

pub const EXTRA_FILES_DIR_NAME: &str = "extra_files";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
}

pub type ReleaseAssets = HashMap<ArchOs, Asset>;

#[derive(Debug, Clone, Default)]
pub struct Release {
    pub added_at: Option<DateTime<Utc>>,
    pub assets: ReleaseAssets,
}

impl Release {
    pub fn with_assets(mut self, assets: ReleaseAssets) -> Self {
        self.assets = assets;
        self
    }

    pub fn with_added_at(mut self, added_at: Option<DateTime<Utc>>) -> Self {
        self.added_at = added_at;
        self
    }
}

fn is_zero(x: &u32) -> bool {
    *x == 0
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Install {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub strip: u32,
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_files: BTreeMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub repository: String,
    pub comment: String,
    pub releases: BTreeMap<Version, Release>,

    pub installs: BTreeMap<Version, HashMap<ArchOs, Install>>,
    pub package_dir: PathBuf,

    pub fetcher: FetcherConfig,
}

impl Package {
    pub fn from_file(path: &Path) -> Result<Package> {
        let file = File::open(path)?;
        let internal_package: InternalPackage = serde_yaml::from_reader(file)?;
        let package_dir = path
            .parent()
            .ok_or_else(|| anyhow!("No parent dir for package {}", path.display()))?;
        internal_package.to_package(package_dir)
    }

    pub fn from_yaml_str(yaml_str: &str) -> Result<Package> {
        let internal_package: InternalPackage = serde_yaml::from_str(yaml_str)?;
        internal_package.to_package(&PathBuf::new())
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let internal_package = InternalPackage::from(self);
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
            comment: self.comment.clone(),
            releases,
            installs: self.installs.clone(),
            package_dir: self.package_dir.clone(),
            fetcher: self.fetcher.clone(),
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
        let asset = release.assets.get(arch_os);
        if asset.is_some() {
            return asset;
        }
        if arch_os.arch != Arch::Any {
            let asset = release.assets.get(&arch_os.with_any_arch());
            if asset.is_some() {
                return asset;
            }
        }
        if arch_os.os != Os::Any {
            let asset = release.assets.get(&arch_os.with_any_os());
            if asset.is_some() {
                return asset;
            }
        }
        release.assets.get(&ArchOs::any())
    }

    /// Return files definition for wanted_version
    /// Uses the highest version which is less or equal to wanted_version
    pub fn get_install(&self, wanted_version: &Version, arch_os: &ArchOs) -> Option<&Install> {
        let install = self.get_install_internal(wanted_version, arch_os);
        if install.is_some() {
            return install;
        }
        if arch_os.arch != Arch::Any {
            let install = self.get_install_internal(wanted_version, &arch_os.with_any_arch());
            if install.is_some() {
                return install;
            }
        }
        if arch_os.os != Os::Any {
            // Probably less useful than the previous check, but you never know
            let install = self.get_install_internal(wanted_version, &arch_os.with_any_os());
            if install.is_some() {
                return install;
            }
        }
        self.get_install_internal(wanted_version, &ArchOs::any())
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

    /// Return a copy of self without releases that were added less than `cooldown_days` ago
    pub fn enforce_cooldown_days(&self, cooldown_days: usize) -> Self {
        let mut package = self.clone();

        let max_added_at = Utc::now() - TimeDelta::days(cooldown_days as i64);
        package.releases.retain(|&_, r| match r.added_at {
            None => true,
            Some(added_at) => added_at <= max_added_at,
        });
        package
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use std::{fs, str::FromStr};

    use crate::store::INDEX_NAME;

    const TEST_PACKAGE_YAML_CONTENT: &str = "
    name: test
    description: desc
    homepage:
    releases:
      1.2.0:
        any:
          url: https://example.com/foo-1.2.0
          sha256: '1234'
      1.3.0:
        added_at: '2024-01-02T12:34:56Z'
        assets:
          any:
            url: https://example.com/foo-1.3.0
            sha256: '5678'
    installs:
      1.2.0:
        any:
          strip: 1
          files:
            bin/foo-1.2: bin/foo
            share:
          tests:
            - foo --help
            - foo --version
    fetcher: !GitHub
      arch: x86_64
      os: linux
    ";

    /// Helper function to read a YAML file and return the Mapping representing it
    fn read_yaml_from_path(path: &Path) -> serde_yaml::Mapping {
        let file = File::open(path).unwrap();
        let value: serde_yaml::Value = serde_yaml::from_reader(file).unwrap();
        value.as_mapping().unwrap().clone()
    }

    #[test]
    fn from_file_should_load_packages_defined_as_dirs() {
        // GIVEN a package defined as a dir
        let dir = assert_fs::TempDir::new().unwrap();
        let package_dir = dir.join("test");
        fs::create_dir(&package_dir).unwrap();

        let package_file = package_dir.join(INDEX_NAME);
        fs::write(&package_file, TEST_PACKAGE_YAML_CONTENT).unwrap();

        // WHEN loading the package
        // THEN it is loaded as expected
        let package = Package::from_file(&package_file).unwrap();

        // AND its package_dir is correct
        assert_eq!(package.package_dir, package_dir);

        // AND its tests are valid
        let version = Version::new(1, 2, 0);
        let install = package.get_install(&version, &ArchOs::current()).unwrap();
        assert_eq!(install.tests, &["foo --help", "foo --version"]);
    }

    #[test]
    fn from_file_should_load_packages_defined_as_files() {
        // GIVEN a package defined as a file
        let dir = assert_fs::TempDir::new().unwrap();

        let package_file = dir.join("test.yaml");
        fs::write(&package_file, TEST_PACKAGE_YAML_CONTENT).unwrap();

        // WHEN loading the package
        // THEN it is loaded as expected
        let package = Package::from_file(&package_file).unwrap();

        // AND its package_dir is correct
        assert_eq!(package.package_dir, dir.path());

        // AND its tests are valid
        let version = Version::new(1, 2, 0);
        let install = package.get_install(&version, &ArchOs::current()).unwrap();
        assert_eq!(install.tests, &["foo --help", "foo --version"]);
    }

    #[test]
    fn test_loading_package() {
        // GIVEN a package defined by TEST_PACKAGE_YAML_CONTENT
        // WHEN its loaded
        let package = Package::from_yaml_str(TEST_PACKAGE_YAML_CONTENT).unwrap();

        // THEN the 1.2.0 release, which uses the V1 variant, is correctly loaded
        let release_120 = package
            .releases
            .get(&Version::from_str("1.2.0").unwrap())
            .unwrap();
        assert!(release_120.added_at.is_none());
        let asset_120 = release_120.assets.get(&ArchOs::any()).unwrap();
        assert_eq!(asset_120.sha256, "1234");

        // AND the 1.3.0 release, which uses the V2 variant, is correctly loaded
        let release_130 = package
            .releases
            .get(&Version::from_str("1.3.0").unwrap())
            .unwrap();
        assert_eq!(
            release_130.added_at,
            Some(
                DateTime::parse_from_rfc3339("2024-01-02T12:34:56Z")
                    .unwrap()
                    .to_utc()
            )
        );
        assert!(release_130.assets.contains_key(&ArchOs::any()));
        let asset_130 = release_130.assets.get(&ArchOs::any()).unwrap();
        assert_eq!(asset_130.sha256, "5678");

        // AND the install section for the 1.2.0 release is correctly loaded
        let install = package
            .get_install(&Version::new(1, 2, 0), &ArchOs::current())
            .unwrap();
        assert_eq!(
            install.files.get("bin/foo-1.2"),
            Some(&"bin/foo".to_string())
        );
        assert_eq!(install.files.get("share"), Some(&"".to_string()));
    }

    #[test]
    fn saving_package_must_write_correct_release_format() {
        // GIVEN a package
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
              2.0.0:
                x86_64-linux:
                  url: https://example.com
                  sha256: '1234'
            installs: {}
            ",
        )
        .unwrap();

        // WHEN it's saved to disk
        let dir = assert_fs::TempDir::new().unwrap();
        let path = dir.join("test.yaml");
        package.to_file(&path).unwrap();

        // THEN it uses the correct format for releases
        let root = read_yaml_from_path(&path);

        // Get the 2.0.0 release
        let release = root
            .get("releases")
            .unwrap()
            .as_mapping()
            .unwrap()
            .get("2.0.0")
            .unwrap()
            .as_mapping()
            .unwrap();

        // The release must be using the V2 format, with "assets" and "added_at" keys
        let mut keys: Vec<String> = release
            .keys()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        keys.sort();
        assert_eq!(keys, &["added_at", "assets"]);
    }

    #[test]
    fn saving_package_keeps_comment() {
        // GIVEN a package with a comment
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            comment: Careful with test
            releases:
              2.0.0:
                x86_64-linux:
                  url: https://example.com
                  sha256: '1234'
            installs: {}
            ",
        )
        .unwrap();

        // WHEN it's saved to disk
        let dir = assert_fs::TempDir::new().unwrap();
        let path = dir.join("test.yaml");
        package.to_file(&path).unwrap();

        // THEN the comment is kept
        let root = read_yaml_from_path(&path);
        let comment = root.get("comment").unwrap();
        assert_eq!(comment.as_str().unwrap(), "Careful with test");
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
                  sha256: '1234'
              1.2.1:
                any:
                  url: https://example.com
                  sha256: '1234'
              1.2.0:
                any:
                  url: https://example.com
                  sha256: '1234'
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
            .get_install(
                &Version::new(1, 0, 0),
                &ArchOs::new(Arch::X86_64, Os::MacOs),
            )
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
            .get_install(&Version::new(1, 0, 0), &ArchOs::any())
            .unwrap();
        assert_eq!(install.strip, 0);
    }

    #[test]
    fn enforce_cooldown_days_remove_too_recent_release() {
        // GIVEN a package with release 2.0 from 2 day ago
        // AND release 1.0 from 4 days ago
        let now = Utc::now();
        let v1_added_at = now - TimeDelta::days(4);
        let v2_added_at = now - TimeDelta::days(2);
        let package = Package::from_yaml_str(&format!(
            "
            name: foo
            description: The foo package
            homepage:
            releases:
              1.0.0:
                added_at: {}
                assets:
                  any:
                    url: https://example.com/foo
                    sha256: '1234'
              2.0.0:
                added_at: {}
                assets:
                  any:
                    url: https://example.com/foo
                    sha256: '1234'
            installs: {{}}
            ",
            v1_added_at, v2_added_at
        ))
        .unwrap();

        // WHEN enforce_cooldown_days(3) is called
        let package = package.enforce_cooldown_days(3);

        // THEN only release 1.0 is kept
        let versions: Vec<Version> = package.releases.keys().cloned().collect();
        assert_eq!(versions, &[Version::new(1, 0, 0)]);
    }
}
