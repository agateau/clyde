// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

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
