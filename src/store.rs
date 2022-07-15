// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::package::Package;

#[derive(Clone)]
pub struct SearchHit {
    pub name: String,
    pub description: String,
}

pub trait Store {
    fn setup(&self) -> Result<()>;
    fn update(&self) -> Result<()>;
    fn get_package(&self, name: &str) -> Result<Package>;
    fn search(&self, query: &str) -> Result<Vec<SearchHit>>;
}

pub struct GitStore {
    url: String,
    dir: PathBuf,
}

impl SearchHit {
    fn from_package(package: &Package) -> SearchHit {
        SearchHit {
            name: package.name.clone(),
            description: package.description.clone(),
        }
    }
}

impl GitStore {
    pub fn new(url: &str, dir: &Path) -> GitStore {
        GitStore {
            url: url.to_string(),
            dir: dir.to_path_buf(),
        }
    }

    fn find_package_path(&self, name: &str) -> Option<PathBuf> {
        let direct_path = PathBuf::from(name);
        if direct_path.is_file() {
            return Some(direct_path);
        }
        let store_path = self.dir.join(name.to_owned() + ".yaml");
        if store_path.is_file() {
            return Some(store_path);
        }
        None
    }
}

impl Store for GitStore {
    fn setup(&self) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.args(["clone", &self.url]);
        cmd.arg(self.dir.as_os_str());

        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Failed to clone Clyde store"));
        }
        Ok(())
    }

    fn update(&self) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("-C");
        cmd.arg(self.dir.as_os_str());
        cmd.arg("pull");
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Failed to update"));
        }
        Ok(())
    }

    fn get_package(&self, name: &str) -> Result<Package> {
        let path = self
            .find_package_path(name)
            .ok_or_else(|| anyhow!("No such package: {}", name))?;
        Package::from_file(&path)
    }

    fn search(&self, query_: &str) -> Result<Vec<SearchHit>> {
        let query = query_.to_lowercase();
        let mut name_hits = Vec::<SearchHit>::new();
        let mut description_hits = Vec::<SearchHit>::new();

        // This implementation is very inefficient, but it's good enough for now given the number
        // of available packages. It should be revisited when the number of packages grow.
        // A possible solution is to create a database table to store the name and descriptions of
        // available packages.
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension() != Some(&OsString::from("yaml")) {
                continue;
            }
            let package = Package::from_file(&path).expect("Skipping invalid package");

            if package.name.to_lowercase().contains(&query) {
                name_hits.push(SearchHit::from_package(&package));
            } else if package.description.to_lowercase().contains(&query) {
                description_hits.push(SearchHit::from_package(&package));
            }
        }

        name_hits.extend_from_slice(&description_hits);
        Ok(name_hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::vec::Vec;

    fn create_package_file(dir: &Path, name: &str) {
        create_package_file_with_desc(dir, name, &format!("The {} package", name));
    }

    fn create_package_file_with_desc(dir: &Path, name: &str, desc: &str) {
        let path = dir.join(name.to_owned() + ".yaml");
        fs::write(
            path,
            format!(
                "
        name: {name}
        description: {desc}
        homepage:
        releases: {{}}
        installs: {{}}
        ",
                name = name,
                desc = desc
            ),
        )
        .unwrap();
    }

    #[test]
    fn search_should_find_packages() {
        let dir = assert_fs::TempDir::new().unwrap();

        // GIVEN a store with 3 packages foo, bar and baz (whose description contains Foo)
        let store = GitStore::new("https://example.com", &dir);

        create_package_file(&dir, "foo");
        create_package_file(&dir, "bar");
        create_package_file_with_desc(&dir, "baz", "Helper package for Foo");

        // WHEN I search for bar
        let results = store.search("fOo").unwrap();

        // THEN foo and baz should be returned
        let result_names: Vec<String> = results.iter().map(|x| x.name.clone()).collect();

        assert_eq!(result_names, vec!["foo", "baz"]);
    }
}
