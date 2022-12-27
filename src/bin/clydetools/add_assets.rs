// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::slice::Iter;

use anyhow::{anyhow, Result};
use regex::Regex;
use semver::Version;

use clyde::app::App;
use clyde::arch_os::{Arch, ArchOs, Os};
use clyde::checksum::compute_checksum;
use clyde::file_cache::FileCache;
use clyde::package::{Asset, Package, Release};
use clyde::ui::Ui;

lazy_static! {
    // Order matters: x86_64 must be looked for before x86
    static ref ARCH_VEC: Vec<(&'static str, Arch)> = vec![
        ("x86_64", Arch::X86_64),
        ("amd64", Arch::X86_64),
        ("x64", Arch::X86_64),
        ("x86", Arch::X86),
        ("i?[36]86", Arch::X86),
        ("aarch64", Arch::Aarch64),
        ("arm64", Arch::Aarch64),
        ("32bit", Arch::X86),
        ("64bit", Arch::X86_64),
        ("universal", Arch::Any),
    ];
    static ref OS_VEC: Vec<(&'static str, Os)> = vec![
        ("linux", Os::Linux),
        ("darwin", Os::MacOs),
        ("apple", Os::MacOs),
        ("macos(|10|11)", Os::MacOs),
        ("mac", Os::MacOs),
        ("win(|dows|32|64)", Os::Windows),
    ];
    static ref UNSUPPORTED_EXTS : HashSet<&'static str> = HashSet::from(["deb", "rpm", "msi", "apk", "asc", "sha256", "sbom", "txt", "dmg", "sh"]);

    // Packer extensions, ordered from worse to best
    static ref PACKER_EXTENSIONS: Vec<&'static str> = vec![
        "zip", "gz", "bz2", "xz"
    ];

    // Libc names, ordered from worse to best. List msvc at the end because on Windows it tends to
    // produce smaller binaries
    static ref LIBC_NAMES: Vec<&'static str> = vec![
        "gnu", "musl", "msvc"
    ];
}

const ARCH_OS_SEPARATOR_PATTERN: &str = "(\\b|[-_.])";

fn compute_url_checksum(ui: &Ui, cache: &FileCache, url: &str) -> Result<String> {
    let path = cache.download(ui, url)?;
    ui.info("Computing checksum");
    compute_checksum(&path)
}

// Must take an iterator as argument because each *_VEC is a unique type
fn find_in_iter<T: Copy>(iter: Iter<'_, (&'static str, T)>, name: &str) -> Option<T> {
    for (pattern, key) in iter {
        let pattern = format!("{ARCH_OS_SEPARATOR_PATTERN}{pattern}{ARCH_OS_SEPARATOR_PATTERN}");
        let rx = Regex::new(&pattern).unwrap();
        if rx.is_match(name) {
            return Some(*key);
        }
    }
    None
}

fn extract_arch_os(
    name: &str,
    default_arch: Option<Arch>,
    default_os: Option<Os>,
) -> Option<ArchOs> {
    let arch = find_in_iter(ARCH_VEC.iter(), name).or(default_arch)?;
    let os = find_in_iter(OS_VEC.iter(), name).or(default_os)?;
    Some(ArchOs::new(arch, os))
}

fn get_extension(name: &str) -> Option<&str> {
    name.rsplit_once('.').map(|x| x.1)
}

fn get_lname(url: &str) -> Result<String> {
    let (_, name) = url
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;
    Ok(name.to_ascii_lowercase())
}

fn is_supported_name(name: &str) -> bool {
    let ext = match get_extension(name) {
        Some(x) => x,
        None => return true,
    };
    if UNSUPPORTED_EXTS.contains(ext) {
        return false;
    }
    true
}

pub fn add_asset(
    ui: &Ui,
    cache: &FileCache,
    release: &mut Release,
    arch_os: &ArchOs,
    url: &str,
) -> Result<()> {
    let checksum = compute_url_checksum(ui, cache, url)?;

    let asset = Asset {
        url: url.to_string(),
        sha256: checksum,
    };

    release.insert(arch_os.clone(), asset);

    Ok(())
}

/// Given an asset URL, return a score based on the packer it uses. The better the packer, the
/// higher the score.
fn get_pack_score(name: &str) -> usize {
    let ext = match get_extension(name) {
        Some(x) => x,
        None => return 0,
    };

    match PACKER_EXTENSIONS.iter().position(|&x| x == ext) {
        Some(idx) => idx + 1,
        None => 0,
    }
}

/// Given an asset URL, return a score based on the libc it uses. The better the libc, the higher
/// the score.
fn get_libc_score(name: &str) -> usize {
    match LIBC_NAMES.iter().position(|&x| name.contains(x)) {
        Some(idx) => idx + 1,
        None => 0,
    }
}

/// Given two asset URLs, return the one we prefer to use
fn select_best_url<'a>(ui: &Ui, u1: &'a str, u2: &'a str) -> &'a str {
    // We know get_lname() is going to succeed here, because it has already been called before
    let n1 = get_lname(u1).unwrap();
    let n2 = get_lname(u2).unwrap();

    let u1_libc_score = get_libc_score(&n1);
    let u2_libc_score = get_libc_score(&n2);
    match u1_libc_score.cmp(&u2_libc_score) {
        Ordering::Greater => return u1,
        Ordering::Less => return u2,
        Ordering::Equal => (),
    };

    let u1_pack_score = get_pack_score(&n1);
    let u2_pack_score = get_pack_score(&n2);
    match u1_pack_score.cmp(&u2_pack_score) {
        Ordering::Greater => u1,
        Ordering::Less => u2,
        Ordering::Equal => {
            ui.warn(&format!(
                "Don't know which of '{n1}' and '{n2}' is the best, picking the first one"
            ));
            u1
        }
    }
}

/// Given a bunch of asset URLs returns the best URL per arch-os
pub fn select_best_urls(
    ui: &Ui,
    urls: &Vec<String>,
    default_arch: Option<Arch>,
    default_os: Option<Os>,
) -> Result<HashMap<ArchOs, String>> {
    let mut best_urls = HashMap::<ArchOs, String>::new();
    for url in urls {
        let lname = get_lname(url)?;
        if !is_supported_name(&lname) {
            continue;
        }

        let arch_os = match extract_arch_os(&lname, default_arch, default_os) {
            Some(x) => x,
            None => {
                ui.warn(&format!("Can't extract arch-os from {lname}, skipping"));
                continue;
            }
        };
        let url = match best_urls.get(&arch_os) {
            Some(current_url) => select_best_url(ui, current_url, url),
            None => url,
        };
        best_urls.insert(arch_os, url.to_string());
    }
    Ok(best_urls)
}

pub fn add_assets(
    app: &App,
    ui: &Ui,
    path: &Path,
    version: &Version,
    arch_os: &Option<String>,
    urls: &Vec<String>,
) -> Result<()> {
    let package = Package::from_file(path)?;

    let mut release = match package.releases.get(version) {
        Some(x) => x.clone(),
        None => Release::new(),
    };

    if let Some(arch_os) = arch_os {
        if urls.len() > 1 {
            return Err(anyhow!("When using --arch-os, only one URL can be passed"));
        }
        let url = urls.first().unwrap();
        let arch_os = ArchOs::parse(arch_os)?;
        add_asset(ui, &app.download_cache, &mut release, &arch_os, url)?;
    } else {
        let urls_for_arch_os = select_best_urls(ui, urls, None, None)?;
        for (arch_os, url) in urls_for_arch_os {
            ui.info(&format!("{arch_os}: {url}"));
            let result = add_asset(
                &ui.nest(),
                &app.download_cache,
                &mut release,
                &arch_os,
                &url,
            );
            if let Err(err) = result {
                ui.error(&format!(
                    "Can't add {:?} build from {}: {}",
                    arch_os, url, err
                ));
                return Err(err);
            };
        }
    }

    let new_package = package.replace_release(version, release);
    new_package.to_file(path)?;

    Ok(())
}

/// Wraps add_assets to make it easier to use as a standalone command
pub fn add_assets_cmd(
    app: &App,
    ui: &Ui,
    path: &Path,
    version: &str,
    arch_os: &Option<String>,
    urls: &Vec<String>,
) -> Result<()> {
    let version = Version::parse(version)?;
    add_assets(app, ui, path, &version, arch_os, urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use clyde::test_file_utils::get_fixture_path;

    fn check_extract_arch_os(filename: &str, expected: Option<ArchOs>) {
        let result = extract_arch_os(filename, None, None);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_arch_os_from_store() {
        let yaml_path = get_fixture_path("store_arch_os.csv");
        let content = fs::read(yaml_path).unwrap();
        let content = String::from_utf8(content).unwrap();
        let mut ok = true;
        for line in content.lines() {
            let (name, arch_os_str) = line.trim().split_once(",").unwrap();
            let expected = ArchOs::parse(arch_os_str).unwrap();

            let result = extract_arch_os(name, None, None);
            if result != Some(expected) {
                eprintln!("Failure: name={} arch_os_str={}", name, arch_os_str);
                ok = false;
            }
        }
        assert!(ok);
    }

    #[test]
    fn test_extract_arch_os() {
        check_extract_arch_os(
            "foo-1.2-linux-arm64.tar.gz",
            Some(ArchOs::new(Arch::Aarch64, Os::Linux)),
        );
        check_extract_arch_os(
            "node-v16.16.0-win-x86.zip",
            Some(ArchOs::new(Arch::X86, Os::Windows)),
        );
        check_extract_arch_os(
            "node-v16.16.0-darwin-x64.tar.gz",
            Some(ArchOs::new(Arch::X86_64, Os::MacOs)),
        );
        check_extract_arch_os(
            "bat-v0.21.0-i686-pc-windows-msvc.zip",
            Some(ArchOs::new(Arch::X86, Os::Windows)),
        );
        check_extract_arch_os(
            "cmake-3.24.0-rc5-macos10.10-universal.tar.gz",
            Some(ArchOs::new(Arch::Any, Os::MacOs)),
        );
        check_extract_arch_os("bar-3.14.tar.gz", None);
    }

    #[test]
    fn test_extract_arch_os_default_values() {
        let result = extract_arch_os("ninja-windows.zip", Some(Arch::X86_64), None);
        assert_eq!(result, Some(ArchOs::new(Arch::X86_64, Os::Windows)));

        let result = extract_arch_os("ninja-mac.zip", Some(Arch::X86_64), None);
        assert_eq!(result, Some(ArchOs::new(Arch::X86_64, Os::MacOs)));
    }

    #[test]
    fn test_is_supported_name() {
        assert!(is_supported_name("foo.tar.gz"));
        assert!(is_supported_name("foo.zip"));
        assert!(is_supported_name("foo.exe"));
        assert!(is_supported_name("foo-x86_64-linux"));
        assert!(is_supported_name("foo.gz"));
        assert!(is_supported_name("foo.exe.xz"));
        assert!(is_supported_name("foo.bz2"));

        assert!(!is_supported_name("foo.deb"));
        assert!(!is_supported_name("foo.rpm"));
        assert!(!is_supported_name("foo.msi"));
    }

    #[test]
    fn test_select_best_url() {
        let ui = Ui::default();

        assert_eq!(
            select_best_url(&ui, "https://example.com/foo.gz", "https://example.com/foo"),
            "https://example.com/foo.gz"
        );
        assert_eq!(
            select_best_url(
                &ui,
                "https://example.com/foo.gz",
                "https://example.com/foo.xz"
            ),
            "https://example.com/foo.xz"
        );

        // libc is more important than compression
        assert_eq!(
            select_best_url(
                &ui,
                "https://example.com/foo-musl",
                "https://example.com/foo-glibc.gz"
            ),
            "https://example.com/foo-musl"
        );
    }
}
