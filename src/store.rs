// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::package::Package;

const INDEX_NAME: &str = "index.yaml";

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
        if name.ends_with(".yaml") {
            let direct_path = PathBuf::from(name);
            return if direct_path.is_file() {
                Some(direct_path)
            } else {
                None
            };
        }
        let store_path = self.dir.join(name).join(INDEX_NAME);
        if store_path.is_file() {
            return Some(store_path);
        }
        let store_path = self.dir.join(name.to_owned() + ".yaml");
        if store_path.is_file() {
            return Some(store_path);
        }
        None
    }
}

/// Return path to the YAML file if it exists
/// If path is a dir, returns <path>/index.yaml
/// If path is a file, returns <path> if its extension is .yaml
fn get_package_path(path: &Path) -> Option<PathBuf> {
    if path.is_dir() {
        let path = path.join(INDEX_NAME);
        if path.exists() {
            return Some(path);
        } else {
            return None;
        }
    }
    if path.extension() != Some(&OsString::from("yaml")) {
        return None;
    }
    Some(path.into())
}

impl Store for GitStore {
    fn setup(&self) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.args(["clone", &self.url]);
        cmd.arg(self.dir.as_os_str());

        let status = match cmd.status() {
            Ok(x) => x,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Err(anyhow!("Failed to clone Clyde store: git is not installed"));
                } else {
                    return Err(err.into());
                }
            }
        };
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
        // A possible solution is to create a database table to store the name and description of
        // available packages.
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            let path = match get_package_path(&path) {
                Some(x) => x,
                None => continue,
            };
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

    use crate::test_file_utils::CwdSaver;

    fn create_package_file(dir: &Path, name: &str) {
        create_package_file_with_desc(dir, name, &format!("The {} package", name));
    }

    fn create_package_file_with_desc(dir: &Path, name: &str, desc: &str) {
        let package_dir = dir.join(name);
        fs::create_dir(&package_dir).unwrap();
        let path = package_dir.join(INDEX_NAME);
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

    fn create_old_package_file_with_desc(dir: &Path, name: &str, desc: &str) {
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
        create_old_package_file_with_desc(&dir, "baz", "Helper package for Foo");

        // WHEN I search for fOo
        let results = store.search("fOo").unwrap();

        // THEN foo and baz should be returned
        let result_names: Vec<String> = results.iter().map(|x| x.name.clone()).collect();

        assert_eq!(result_names, vec!["foo", "baz"]);
    }

    #[test]
    fn find_package_path_should_not_try_to_read_files_without_yaml_extensions() {
        // GIVEN an empty store
        let dir = assert_fs::TempDir::new().unwrap();
        let store_dir = dir.join("store");
        let store = GitStore::new("https://example.com", &store_dir);

        // AND a file called foo in the current dir
        fs::write(dir.join("foo"), "").unwrap();
        let _cwd_saver = CwdSaver::cd(&dir);
        assert!(PathBuf::from("foo").exists());

        // WHEN find_package_path("foo") is called
        let path = store.find_package_path("foo");

        // THEN it should return None
        assert_eq!(path, None);
    }
}
