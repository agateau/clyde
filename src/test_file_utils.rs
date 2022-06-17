use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::vec::Vec;

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

fn list_tree_internal(root: &Path, parent: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::<PathBuf>::new();
    for entry in fs::read_dir(&parent)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(list_tree_internal(root, &path)?);
        } else {
            let rel_path = path.strip_prefix(&root)?;
            files.push(rel_path.to_path_buf());
        }
    }
    Ok(files)
}

pub fn list_tree(root: &Path) -> Result<Vec<String>> {
    let file_vec = list_tree_internal(root, root)?;
    let file_array = file_vec
        .iter()
        .map(|p| p.to_str().unwrap().to_string())
        .collect();
    Ok(file_array)
}
