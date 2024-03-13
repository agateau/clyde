// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::vec::Vec;

use anyhow::Result;
use semver::{Version, VersionReq};

use crate::app::App;
use crate::cmd::{install_packages, InstallRequest};
use crate::db::{Database, PackageInfo};
use crate::package::Package;
use crate::store::Store;
use crate::ui::Ui;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Upgrade {
    pub package_info: PackageInfo,
    pub available_version: Version,
}

impl Upgrade {
    fn new(package_info: &PackageInfo, available_version: &Version) -> Self {
        Upgrade {
            package_info: package_info.clone(),
            available_version: available_version.clone(),
        }
    }
}

fn get_newer_version(
    package: &Package,
    installed_version: &Version,
    version_req: &VersionReq,
) -> Option<Version> {
    if let Some(version) = package.get_version_matching(version_req) {
        if version > installed_version {
            return Some(version.clone());
        }
    }
    None
}

/// Check for upgrades, return a couple of Upgrade vectors. First element contains installable
/// upgrades, second element contains blocked upgrades.
fn get_upgrades(ui: &Ui, store: &dyn Store, db: &Database) -> Result<(Vec<Upgrade>, Vec<Upgrade>)> {
    let mut upgrades = Vec::<Upgrade>::new();
    let mut blocked_upgrades = Vec::<Upgrade>::new();

    for info in db.get_installed_packages()? {
        let package = match store.get_package(&info.name) {
            Ok(x) => x,
            Err(x) => {
                ui.warn(&format!("Can't check updates for {}: {}", info.name, x));
                continue;
            }
        };
        if let Some(available_version) =
            get_newer_version(&package, &info.installed_version, &info.requested_version)
        {
            upgrades.push(Upgrade::new(&info, &available_version));
        } else if let Some(available_version) =
            get_newer_version(&package, &info.installed_version, &VersionReq::STAR)
        {
            blocked_upgrades.push(Upgrade::new(&info, &available_version));
        }
    }
    Ok((upgrades, blocked_upgrades))
}

pub fn upgrade_cmd(app: &App, ui: &Ui) -> Result<()> {
    ui.info("Checking upgrades");
    let (upgrades, blocked_upgrades) = get_upgrades(&ui.nest(), &*app.store, &app.database)?;

    if !blocked_upgrades.is_empty() {
        ui.info("Blocked upgrades:");
        for upgrade in blocked_upgrades {
            ui.println(&format!(
                "- {}: can be upgraded to {}, but pinned to {}",
                upgrade.package_info.name,
                upgrade.available_version,
                upgrade.package_info.requested_version
            ));
        }
    }

    if upgrades.is_empty() {
        ui.info("No packages to upgrade");
        return Ok(());
    }

    ui.info("Available upgrades:");
    for upgrade in &upgrades {
        ui.println(&format!(
            "- {}: {} → {}",
            upgrade.package_info.name,
            upgrade.package_info.installed_version,
            upgrade.available_version
        ));
    }

    let install_requests = upgrades
        .iter()
        .map(|u| {
            InstallRequest::new(
                &u.package_info.name,
                u.package_info.requested_version.clone(),
            )
        })
        .collect();
    install_packages(app, ui, false /* reinstall */, &install_requests)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use anyhow::{anyhow, Error};
    use semver::{Version, VersionReq};

    use crate::package::Package;
    use crate::store::SearchHit;

    struct FakeStore {
        packages: HashMap<String, Package>,
    }

    impl FakeStore {
        fn new() -> FakeStore {
            FakeStore {
                packages: HashMap::<String, Package>::new(),
            }
        }
    }

    impl Store for FakeStore {
        fn setup(&self, _url: &str) -> Result<()> {
            Ok(())
        }
        fn update(&self) -> Result<()> {
            Ok(())
        }
        fn get_package(&self, name: &str) -> Result<Package> {
            let pkg = self
                .packages
                .get(name)
                .ok_or_else(|| anyhow!("No such package: {}", name))?;
            Ok(pkg.clone())
        }
        fn search(&self, _query: &str) -> Result<(Vec<SearchHit>, Vec<Error>)> {
            Ok((vec![], vec![]))
        }
    }

    #[test]
    fn get_upgrades_should_return_an_empty_list_if_nothing_to_do() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // AND a package foo at version 1.2.0
        let files = HashSet::<PathBuf>::new();
        db.add_package("foo", &Version::new(1, 2, 0), &VersionReq::STAR, &files)
            .unwrap();

        // AND a store with package foo at version 1.2.0
        let mut store = FakeStore::new();
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
              1.2.0:
                any:
                  url: https://example.com
                  sha256: 1234
            installs: {}
            ",
        )
        .unwrap();
        store.packages.insert("foo".to_string(), package);

        // WHEN get_upgrades() is called
        let (upgrades, blocked_upgrades) = get_upgrades(&Ui::default(), &store, &db).unwrap();

        // THEN it returns empty vectors
        assert!(upgrades.is_empty());
        assert!(blocked_upgrades.is_empty());
    }

    #[test]
    fn get_upgrades_should_return_a_blocked_upgrade_if_upgrade_is_outside_requested_version() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // AND a package foo at version 1.2.0, pinned to 1.2.*
        let files = HashSet::<PathBuf>::new();
        let version_req = VersionReq::parse("1.2.*").unwrap();
        db.add_package("foo", &Version::new(1, 2, 0), &version_req, &files)
            .unwrap();

        // AND a store with package foo at version 1.2.0 and 1.3.0
        let mut store = FakeStore::new();
        let package = Package::from_yaml_str(
            "
            name: foo
            description: desc
            homepage:
            releases:
              1.2.0:
                any:
                  url: https://example.com
                  sha256: 1234
              1.3.0:
                any:
                  url: https://example.com
                  sha256: 1234
            installs: {}
            ",
        )
        .unwrap();
        store.packages.insert("foo".to_string(), package);

        // WHEN get_upgrades() is called
        let (upgrades, blocked_upgrades) = get_upgrades(&Ui::default(), &store, &db).unwrap();

        // THEN it returns an empty upgrade vector
        assert_eq!(upgrades, vec![]);

        // AND a blocked upgrade in the blocked_upgrades vector
        let package_info = PackageInfo::new("foo", &Version::new(1, 2, 0), &version_req);
        let blocked_upgrade = Upgrade::new(&package_info, &Version::new(1, 3, 0));
        assert_eq!(blocked_upgrades, vec![blocked_upgrade]);
    }

    #[test]
    fn get_upgrades_should_return_upgrade_list() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // AND a package foo at version 1.2.0
        let files = HashSet::<PathBuf>::new();
        db.add_package("foo", &Version::new(1, 2, 0), &VersionReq::STAR, &files)
            .unwrap();

        // AND a store with package foo at version 1.2.0 and 1.3.0
        let mut store = FakeStore::new();
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
              1.2.0:
                any:
                  url: https://example.com
                  sha256: 1234
              1.3.0:
                any:
                  url: https://example.com
                  sha256: 1234
            installs: {}
            ",
        )
        .unwrap();
        store.packages.insert("foo".to_string(), package);

        // WHEN get_upgrades() is called
        let (upgrades, blocked_upgrades) = get_upgrades(&Ui::default(), &store, &db).unwrap();

        // THEN it returns foo
        let package_info = PackageInfo::new("foo", &Version::new(1, 2, 0), &VersionReq::STAR);
        assert_eq!(
            upgrades,
            vec![Upgrade::new(&package_info, &Version::new(1, 3, 0))]
        );
        assert!(blocked_upgrades.is_empty());
    }
}
