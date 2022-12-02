// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use semver::Version;

fn count_chars(txt: &str, wanted: char) -> u32 {
    let mut count = 0;
    for ch in txt.chars() {
        if ch == wanted {
            count += 1;
        }
    }
    count
}

/// Extract a version from a tag
pub fn version_from_tag(tag: &str) -> Result<Version> {
    let version_str = if tag.starts_with('v') {
        tag.get(1..).unwrap()
    } else {
        tag
    };

    let mut version_str = version_str.to_string();

    while count_chars(&version_str, '.') < 2 {
        version_str.push_str(".0");
    }

    let version = Version::parse(&version_str)?;
    Ok(version)
}
