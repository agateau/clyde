// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::vec::Vec;

use anyhow::Result;

use crate::app::App;
use crate::db::{Database, PackageInfo};
use crate::install::install_with_package_and_requested_version;
use crate::store::Store;

fn get_upgrades(store: &dyn Store, db: &Database) -> Result<Vec<PackageInfo>> {
    let mut upgrades = Vec::<PackageInfo>::new();

    for info in db.get_installed_packages()? {
        let package = match store.get_package(&info.name) {
            Ok(x) => x,
            Err(x) => {
                eprintln!("Can't check updates for {}: {}", info.name, x);
                continue;
            }
        };
        if let Some(available_version) = package.get_version_matching(&info.requested_version) {
            if available_version > &info.installed_version {
                upgrades.push(info);
            }
        }
    }
    Ok(upgrades)
}

pub fn upgrade(app: &App) -> Result<()> {
    let to_upgrade = get_upgrades(&*app.store, &app.database)?;
    if to_upgrade.is_empty() {
        eprintln!("No packages to upgrade");
        return Ok(());
    }
    for info in to_upgrade {
        eprintln!("Upgrading {}", info.name);
        install_with_package_and_requested_version(app, &info.name, &info.requested_version)
            .unwrap_or_else(|x| {
                eprintln!("Error: Failed to upgrade {}: {}", info.name, x);
            });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use anyhow::anyhow;
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
        fn setup(&self) -> Result<()> {
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
        fn search(&self, _query: &str) -> Result<Vec<SearchHit>> {
            Ok(vec![])
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
        let upgrades = get_upgrades(&store, &db).unwrap();

        // THEN it returns an empty vector
        assert_eq!(upgrades, vec![]);
    }

    #[test]
    fn get_upgrades_should_return_an_empty_list_if_upgrade_is_outside_requested_version() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // AND a package foo at version 1.2.0, pinned to 1.2.*
        let files = HashSet::<PathBuf>::new();
        db.add_package(
            "foo",
            &Version::new(1, 2, 0),
            &VersionReq::parse("1.2.*").unwrap(),
            &files,
        )
        .unwrap();

        // AND a store with package foo at version 1.3.0
        let mut store = FakeStore::new();
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
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
        let upgrades = get_upgrades(&store, &db).unwrap();

        // THEN it returns an empty vector
        assert_eq!(upgrades, vec![]);
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

        // AND a store with package foo at version 1.3.0
        let mut store = FakeStore::new();
        let package = Package::from_yaml_str(
            "
            name: test
            description: desc
            homepage:
            releases:
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
        let upgrades = get_upgrades(&store, &db).unwrap();

        // THEN it returns foo
        assert_eq!(
            upgrades,
            vec![PackageInfo {
                name: "foo".to_string(),
                installed_version: Version::new(1, 2, 0),
                requested_version: VersionReq::STAR
            }]
        );
    }
}
