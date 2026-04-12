// SPDX-FileCopyrightText: 2026 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::arch_os::ArchOs;
use crate::package::{Asset, FetcherConfig, Install, Package, Release};
use crate::serde_skip::is_empty;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InternalReleaseV2 {
    published_at: Option<DateTime<Utc>>,
    // Key is arch-os
    assets: BTreeMap<String, Asset>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum InternalReleaseEnum {
    V2(InternalReleaseV2),
    // Key is arch-os
    V1(BTreeMap<String, Asset>),
}

fn is_auto_fetcher(x: &FetcherConfig) -> bool {
    *x == FetcherConfig::Auto
}

/// Intermediate struct, used to serialize and deserialize. After deserializing it is turned into
/// Package, which has stronger typing
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InternalPackage {
    pub name: String,
    pub description: String,
    pub homepage: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty")]
    pub comment: String,
    pub releases: Option<BTreeMap<String, InternalReleaseEnum>>,
    pub installs: Option<BTreeMap<String, BTreeMap<String, Install>>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_auto_fetcher")]
    pub fetcher: FetcherConfig,
}

impl From<&Package> for InternalPackage {
    fn from(package: &Package) -> Self {
        let mut releases = BTreeMap::<String, InternalReleaseEnum>::new();
        for (version, release) in package.releases.iter() {
            let version_str = version.to_string();
            let assets: BTreeMap<String, Asset> = release
                .assets
                .iter()
                .map(|(arch_os, asset)| (arch_os.to_str(), asset.clone()))
                .collect();
            /* Uncomment this when we are ready for V2
            let internal_release_v2 = InternalReleaseV2 {
                published_at: release.published_at,
                assets,
            };
            releases.insert(version_str, InternalReleaseEnum::V2(internal_release_v2));
            */
            releases.insert(version_str, InternalReleaseEnum::V1(assets));
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
            comment: package.comment.clone(),
            releases: Some(releases),
            installs: Some(installs),
            fetcher: package.fetcher.clone(),
        }
    }
}

impl InternalPackage {
    pub fn to_package(&self, package_dir: &Path) -> Result<Package> {
        let mut releases = BTreeMap::<Version, Release>::new();

        if let Some(internal_releases) = &self.releases {
            for (version_str, internal_release_enum) in internal_releases.iter() {
                let version = Version::parse(version_str)?;
                let release = match internal_release_enum {
                    InternalReleaseEnum::V2(internal_release) => {
                        let assets = internal_release
                            .assets
                            .iter()
                            .map(|(arch_os, build)| {
                                (ArchOs::parse(arch_os).unwrap(), build.clone())
                            })
                            .collect();
                        Release::default()
                            .with_published_at(internal_release.published_at)
                            .with_assets(assets)
                    }
                    InternalReleaseEnum::V1(internal_release) => {
                        let assets = internal_release
                            .iter()
                            .map(|(arch_os, build)| {
                                (ArchOs::parse(arch_os).unwrap(), build.clone())
                            })
                            .collect();
                        Release::default().with_assets(assets)
                    }
                };
                releases.insert(version, release);
            }
        }

        let mut installs = BTreeMap::<Version, HashMap<ArchOs, Install>>::new();
        if let Some(internal_installs) = &self.installs {
            for (version_str, installs_for_arch_os) in internal_installs.iter() {
                let version = Version::parse(version_str)?;
                let installs_for_arch_os = installs_for_arch_os
                    .iter()
                    .map(|(arch_os, install)| (ArchOs::parse(arch_os).unwrap(), install.clone()))
                    .collect();
                installs.insert(version, installs_for_arch_os);
            }
        }

        Ok(Package {
            name: self.name.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
            repository: self.repository.clone(),
            comment: self.comment.clone(),
            releases,
            installs,
            package_dir: package_dir.to_path_buf(),
            fetcher: self.fetcher.clone(),
        })
    }
}
