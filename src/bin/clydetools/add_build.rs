// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::slice::Iter;

use anyhow::{anyhow, Result};
use semver::Version;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::checksum::compute_checksum;
use clyde::file_cache::FileCache;
use clyde::package::{Build, Package};
use clyde::ui::Ui;

const ARCH_X86_64: &str = "x86_64";
const ARCH_X86: &str = "x86";
const ARCH_AARCH64: &str = "aarch64";

const OS_LINUX: &str = "linux";
const OS_MACOS: &str = "macos";
const OS_WINDOWS: &str = "windows";

type MatchingPair = (&'static str, &'static str);

lazy_static! {
    // Order matters: x86_64 must be looked for before x86
    static ref ARCH_VEC: Vec<MatchingPair> = vec![
        ("x86_64", ARCH_X86_64),
        ("amd64", ARCH_X86_64),
        ("x64", ARCH_X86_64),
        ("x86", ARCH_X86),
        ("386", ARCH_X86),
        ("aarch64", ARCH_AARCH64),
        ("arm64", ARCH_AARCH64),
        ("32bit", ARCH_X86),
        ("64bit", ARCH_X86_64),
    ];
    static ref OS_VEC: Vec<MatchingPair> = vec![
        ("linux", OS_LINUX),
        ("darwin", OS_MACOS),
        ("apple", OS_MACOS),
        ("macos", OS_MACOS),
        ("windows", OS_WINDOWS),
        ("win32", OS_WINDOWS),
        ("win", OS_WINDOWS),
    ];
    static ref UNSUPPORTED_EXTS : HashSet<&'static str> = HashSet::from(["deb", "rpm", "msi"]);
}

fn compute_url_checksum(ui: &Ui, cache: &FileCache, url: &str) -> Result<String> {
    let path = cache.download(ui, url)?;
    compute_checksum(&path)
}

// Must take an iterator as argument because each *_VEC is a unique type
fn find_in_iter(iter: Iter<'_, (&'static str, &'static str)>, name: &str) -> Option<&'static str> {
    for (token, key) in iter {
        if name.contains(token) {
            return Some(key);
        }
    }
    None
}

fn extract_arch_os(name: &str) -> Option<ArchOs> {
    let arch = find_in_iter(ARCH_VEC.iter(), name)?;
    let os = find_in_iter(OS_VEC.iter(), name)?;
    Some(ArchOs::new(arch, os))
}

fn is_supported_name(name: &str) -> bool {
    let ext = match name.rsplit_once('.') {
        Some((_, ext)) => ext,
        None => return true,
    };
    !UNSUPPORTED_EXTS.contains(ext)
}

fn add_build(
    ui: &Ui,
    cache: &FileCache,
    release: &mut HashMap<ArchOs, Build>,
    arch_os: &ArchOs,
    url: &str,
) -> Result<()> {
    let checksum = compute_url_checksum(ui, cache, url)?;

    let build = Build {
        url: url.to_string(),
        sha256: checksum,
    };

    release.insert(arch_os.clone(), build);

    Ok(())
}

pub fn add_builds(
    app: &App,
    ui: &Ui,
    path: &Path,
    version: &str,
    arch_os: &Option<String>,
    urls: &Vec<String>,
) -> Result<()> {
    let package = Package::from_file(path)?;

    let version = Version::parse(version)?;
    let mut release = match package.releases.get(&version) {
        Some(x) => x.clone(),
        None => HashMap::<ArchOs, Build>::new(),
    };

    if let Some(arch_os) = arch_os {
        if urls.len() > 1 {
            return Err(anyhow!("When using --arch-os, only one URL can be passed"));
        }
        let url = urls.first().unwrap();
        let arch_os = ArchOs::parse(arch_os)?;
        add_build(ui, &app.download_cache, &mut release, &arch_os, url)?;
    } else {
        for url in urls {
            let (_, name) = url
                .rsplit_once('/')
                .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;

            let name = name.to_ascii_lowercase();
            if !is_supported_name(&name) {
                ui.info(&format!("Skipping {url}, unsupported extension"));
                continue;
            }

            if let Some(arch_os) = extract_arch_os(&name) {
                if add_build(ui, &app.download_cache, &mut release, &arch_os, url).is_err() {
                    ui.error(&format!("Can't add {:?} build from {}", arch_os, url));
                }
            } else {
                ui.error(&format!("Can't extract arch-os from {}", name));
            }
        }
    }

    let new_package = package.replace_release(&version, release);
    new_package.to_file(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_extract_arch_os(filename: &str, expected: Option<ArchOs>) {
        let result = extract_arch_os(filename);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_arch_os() {
        check_extract_arch_os(
            "foo-1.2-linux-arm64.tar.gz",
            Some(ArchOs::new(ARCH_AARCH64, OS_LINUX)),
        );
        check_extract_arch_os(
            "node-v16.16.0-win-x86.zip",
            Some(ArchOs::new(ARCH_X86, OS_WINDOWS)),
        );
        check_extract_arch_os(
            "node-v16.16.0-darwin-x64.tar.gz",
            Some(ArchOs::new(ARCH_X86_64, OS_MACOS)),
        );
        check_extract_arch_os("bar-3.14.tar.gz", None);
    }

    #[test]
    fn test_is_supported_name() {
        assert!(is_supported_name("foo.tar.gz"));
        assert!(is_supported_name("foo.zip"));
        assert!(is_supported_name("foo.exe"));
        assert!(is_supported_name("foo.gz"));
        assert!(is_supported_name("foo-x86_64-linux"));

        assert!(!is_supported_name("foo.deb"));
        assert!(!is_supported_name("foo.rpm"));
        assert!(!is_supported_name("foo.msi"));
    }
}
