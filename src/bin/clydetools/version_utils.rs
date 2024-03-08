// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Result};
use regex::Regex;
use semver::Version;

lazy_static! {
    static ref VERSION_RX: Regex =
        Regex::new("(?P<version>[0-9]+(\\.[0-9]+)*)(-[0-9]+)?$").unwrap();
}

fn count_chars(txt: &str, wanted: char) -> usize {
    txt.chars().filter(|&x| x == wanted).count()
}

/// Extract a version from a tag
pub fn version_from_tag(tag: &str) -> Result<Version> {
    // Keep only the version suffix
    let version_str = VERSION_RX
        .captures(tag)
        .map(|captures| captures["version"].to_string());

    let mut version_str = match version_str {
        Some(x) => x.to_string(),
        None => {
            return Err(anyhow!("Can't find version number in '{}'", tag));
        }
    };

    // Make sure the version has 3 components
    while count_chars(&version_str, '.') < 2 {
        version_str.push_str(".0");
    }

    let version = Version::parse(&version_str)?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_version_from_tag_removes_prefixes() {
        assert_eq!(version_from_tag("v1.12.0").unwrap(), Version::new(1, 12, 0));
        assert_eq!(
            version_from_tag("foo14-1.12.0").unwrap(),
            Version::new(1, 12, 0)
        );
    }

    #[test]
    fn check_version_from_tag_removes_distro_like_suffix() {
        assert_eq!(
            version_from_tag("forgejo-1.2.3-0").unwrap(),
            Version::new(1, 2, 3)
        );
    }

    #[test]
    fn check_version_from_tag_add_missing_components() {
        assert_eq!(version_from_tag("1").unwrap(), Version::new(1, 0, 0));
        assert_eq!(version_from_tag("1.12").unwrap(), Version::new(1, 12, 0));
    }

    #[test]
    fn check_version_from_tag_fails() {
        assert!(version_from_tag("foo").is_err());
    }
}
