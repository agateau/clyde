// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Result};
use regex::{Regex, RegexBuilder};
use semver::Version;

lazy_static! {
    static ref VERSION_RX: Regex = RegexBuilder::new(
        "(?P<version>
                (
                    # version with possible pre-release info. Must contain at
                    # least two components to avoid ambiguity.
                    [0-9]+(\\.[0-9]+)+(-[a-z.0-9]+)?
                )|(
                    # version with no pre-release info. Supports one component
                    # version numbers.
                    [0-9]+(\\.[0-9]+)*
                )
            )$"
    )
    .ignore_whitespace(true)
    .build()
    .unwrap();
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

    use yare::parameterized;

    #[parameterized(
        skip_prefix1 = { "v1.12.0", "1.12.0"},
        skip_prefix2 = { "foo14-1.12.0", "1.12.0"},
        prerelease1 = { "forgejo-1.2.3-0", "1.2.3-0"},
        prerelease2 = { "v0.2.2-pre", "0.2.2-pre"},
        missing_component1 = {"1", "1.0.0"},
        missing_component2 = {"1.12", "1.12.0"},
    )]
    fn check_version_from_tag_success(tag: &str, expected_str: &str) {
        let expected = Version::parse(expected_str).unwrap();
        assert_eq!(version_from_tag(tag).unwrap(), expected);
    }

    #[test]
    fn check_version_from_tag_fails() {
        assert!(version_from_tag("foo").is_err());
    }
}
