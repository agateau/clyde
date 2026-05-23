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
pub fn check_path_is_safe(path: &Path) -> Result<()> {
    let mut depth: u32 = 0;
    for part in path.components() {
        match part {
            Component::RootDir => {
                return Err(anyhow!(
                    "Malicious path: contains the root dir ({})",
                    path.display()
                ));
            }
            Component::Prefix(..) => {
                return Err(anyhow!(
                    "Malicious path: contains a Windows prefix ({})",
                    path.display()
                ));
            }
            Component::ParentDir => {
                if depth == 0 {
                    return Err(anyhow!(
                        "Malicious path: points outside the archive ({})",
                        path.display()
                    ));
                }
                depth -= 1;
            }
            Component::CurDir => {}
            Component::Normal(_os_str) => {
                depth += 1;
            }
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use yare::parameterized;

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

    #[parameterized(
        file = { "foo", true },
        dir = { "d/f", true },
        dot = { "d/./f", true },
        safe_relative_dir = { "d/../f", true },
        unsafe1 = { "../foo", false },
        // This resolve in "b", so this could be considered safe, but the fact that it goes outside temporarily is suspicious
        unsafe2 = { "d/../../b", false },
        absolute = { "/foo", false },
    )]
    fn check_path_safeness(path: &str, expected: bool) {
        let path = Path::new(path);
        assert_eq!(check_path_is_safe(path).is_ok(), expected)
    }
}
