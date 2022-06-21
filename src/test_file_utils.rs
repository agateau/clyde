use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn create_tree(root: &Path, files: &[&str]) {
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(&path.parent().unwrap()).unwrap();
        File::create(&path).unwrap();
    }
}

pub fn create_tree_from_path_set(root: &Path, files: &HashSet<PathBuf>) {
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(&path.parent().unwrap()).unwrap();
        File::create(&path).unwrap();
    }
}

fn list_tree_internal(root: &Path, parent: &Path) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::<PathBuf>::new();
    for entry in fs::read_dir(&parent)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            for file in list_tree_internal(root, &path)? {
                files.insert(file.to_path_buf());
            }
        } else {
            let rel_path = path.strip_prefix(&root)?;
            files.insert(rel_path.to_path_buf());
        }
    }
    Ok(files)
}

pub fn list_tree(root: &Path) -> Result<HashSet<PathBuf>> {
    list_tree_internal(root, root)
}

pub fn pathbufset_from_strings(strings: &[&str]) -> HashSet<PathBuf> {
    let mut set = HashSet::<PathBuf>::new();
    for string in strings {
        let path = PathBuf::from(string);
        set.insert(path);
    }
    set
}
