// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Saves the current working directory and restores it when dropped
pub struct CwdSaver {
    old_dir: PathBuf,
}

impl CwdSaver {
    pub fn new() -> Self {
        CwdSaver {
            old_dir: env::current_dir().expect("Failed to get current dir"),
        }
    }

    /// Convenience function to save the working directory and change to another one
    pub fn cd(dir: &Path) -> Self {
        let saver = CwdSaver::new();
        env::set_current_dir(dir).unwrap_or_else(|x| panic!("Failed to cd to {dir:?}: {x}"));
        saver
    }
}

impl Default for CwdSaver {
    fn default() -> Self {
        CwdSaver::new()
    }
}

impl Drop for CwdSaver {
    fn drop(&mut self) {
        env::set_current_dir(&self.old_dir).expect("Failed to restore current dir");
    }
}

pub fn create_tree(root: &Path, files: &[&str]) {
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::create(&path).unwrap();
    }
}

pub fn create_tree_from_path_set(root: &Path, files: &HashSet<PathBuf>) {
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        File::create(&path).unwrap();
    }
}

fn list_tree_internal(root: &Path, parent: &Path) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::<PathBuf>::new();
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            for file in list_tree_internal(root, &path)? {
                files.insert(file.to_path_buf());
            }
        } else {
            let rel_path = path.strip_prefix(root)?;
            files.insert(rel_path.to_path_buf());
        }
    }
    Ok(files)
}

pub fn list_tree(root: &Path) -> Result<HashSet<PathBuf>> {
    list_tree_internal(root, root)
}

pub fn pathbufset_from_strings(strings: &[&str]) -> HashSet<PathBuf> {
    strings.iter().map(PathBuf::from).collect()
}

fn get_fixtures_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("src").join("fixtures");
    if !path.exists() {
        panic!("Can't find fixtures dir");
    }
    path
}

pub fn get_fixture_path(name: &str) -> PathBuf {
    let fixtures_dir = get_fixtures_dir();
    let path = fixtures_dir.join(name);
    if !path.exists() {
        panic!("Can't find {name}");
    }
    path
}

#[cfg(unix)]
pub fn is_file_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::metadata(path).unwrap().permissions();
    permissions.mode() & 0o111_u32 == 0o111_u32
}
