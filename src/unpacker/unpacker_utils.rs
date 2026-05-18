// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Result};

pub fn apply_strip(path: &Path, strip: u32) -> Option<PathBuf> {
    if strip == 0 {
        return Some(path.to_owned());
    }
    let prefix = path.iter().next()?;

    let path = path.strip_prefix(prefix).ok()?;
    if path == Path::new("") {
        return None;
    }
    apply_strip(path, strip - 1)
}

/// Prevent unpacking rogue archives: refuse any path that contains ".." or is absolute
pub fn check_path_is_not_malicious(path: &Path) -> Result<()> {
    for part in path.components() {
        match part {
            Component::RootDir => {
                return Err(anyhow!(
                    "Malicious path: contains the root dir ({:?})",
                    path
                ));
            }
            Component::Prefix(..) => {
                return Err(anyhow!(
                    "Malicious path: contains a Windows prefix ({:?})",
                    path
                ));
            }
            Component::ParentDir => {
                return Err(anyhow!("Malicious path: contains \"..\" ({:?})", path));
            }
            _ => {}
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_strip_should_strip_1() {
        assert_eq!(
            apply_strip(Path::new("foo/bar"), 1),
            Some(PathBuf::from("bar"))
        );
    }

    #[test]
    fn apply_strip_should_return_none_when_stripping_too_much() {
        assert_eq!(apply_strip(Path::new("foo/bar"), 2), None);
    }
}
