// SPDX-FileCopyrightText: 2024 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::slice::Iter;

use anyhow::Error as AnyhowError;
use anyhow::{anyhow, Result};
use regex::Regex;

use clyde::arch_os::{Arch, ArchOs, Os};
use clyde::package::FetcherConfig;
use clyde::ui::Ui;

lazy_static! {
    // Order matters: x86_64 must be looked for before x86
    static ref ARCH_VEC: Vec<(&'static str, Arch)> = vec![
        ("x86_64", Arch::X86_64),
        ("amd64", Arch::X86_64),
        ("x64", Arch::X86_64),
        ("x86", Arch::X86),
        ("i?[36]86", Arch::X86),
        ("aarch[-_]?64", Arch::Aarch64),
        ("arm64", Arch::Aarch64),
        ("32[-_]?bit", Arch::X86),
        ("64[-_]?bit", Arch::X86_64),
        ("universal", Arch::Any),
    ];
    static ref OS_VEC: Vec<(&'static str, Os)> = vec![
        ("linux", Os::Linux),
        ("darwin", Os::MacOs),
        ("apple", Os::MacOs),
        ("macos(|10|11)", Os::MacOs),
        ("mac", Os::MacOs),
        ("osx", Os::MacOs),
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

/// Given a bunch of asset URLs returns the best URL per arch-os
pub fn select_best_urls(
    ui: &Ui,
    urls: &[String],
    options: BestUrlOptions,
) -> Result<HashMap<ArchOs, String>> {
    let mut best_urls = HashMap::<ArchOs, ScoredUrl>::new();
    for url in urls {
        let lname = get_lname(url)?;
        if !is_supported_name(&lname) {
            continue;
        }

        if let Some(ref include) = options.include {
            let filename = get_file_name(url)?;
            if !include.is_match(&filename) {
                continue;
            }
        }

        let (arch_os, score) =
            match extract_arch_os(&lname, options.default_arch, options.default_os) {
                Some(x) => x,
                None => {
                    ui.warn(&format!("Can't extract arch-os from {lname}, skipping"));
                    continue;
                }
            };

        let new_scored_url = ScoredUrl::new(url, score);
        best_urls
            .entry(arch_os)
            .and_modify(|e| {
                let best = select_best_url(ui, e, &new_scored_url);
                *e = best.clone();
            })
            .or_insert(new_scored_url);
    }
    Ok(best_urls
        .iter()
        .map(|(arch_os, scored_url)| (*arch_os, scored_url.url.clone()))
        .collect())
}

//- BestUrlOptions --------------------------------------------------
#[derive(Default)]
pub struct BestUrlOptions {
    default_arch: Option<Arch>,
    default_os: Option<Os>,
    include: Option<Regex>,
}

fn parse_include(include: &Option<String>) -> Result<Option<Regex>> {
    match include {
        None => Ok(None),
        Some(expr) => match Regex::new(expr) {
            Ok(rx) => Ok(Some(rx)),
            Err(e) => Err(anyhow!("Failed to parse regular expression {expr}: {e}")),
        },
    }
}

impl BestUrlOptions {
    pub fn new(default_arch: Option<Arch>, default_os: Option<Os>, include: Option<Regex>) -> Self {
        BestUrlOptions {
            default_arch,
            default_os,
            include,
        }
    }
}

impl TryFrom<&FetcherConfig> for BestUrlOptions {
    type Error = AnyhowError;

    fn try_from(config: &FetcherConfig) -> Result<BestUrlOptions> {
        Ok(match config {
            FetcherConfig::Forgejo {
                arch, os, include, ..
            } => BestUrlOptions::new(*arch, *os, parse_include(include)?),
            FetcherConfig::GitHub {
                arch, os, include, ..
            } => BestUrlOptions::new(*arch, *os, parse_include(include)?),
            FetcherConfig::GitLab {
                arch, os, include, ..
            } => BestUrlOptions::new(*arch, *os, parse_include(include)?),
            FetcherConfig::Auto {} => BestUrlOptions::new(None, None, None),
            FetcherConfig::Off {} | FetcherConfig::Script {} => {
                panic!(
                    "BestUrlOptions::try_from should not be called for {:?}",
                    config
                )
            }
        })
    }
}

//- ScoredUrl -------------------------------------------------------
/// Holds an URL and its associated score. Score is higher if the URL does not make use of the
/// default arch or default OS.
#[derive(Debug, PartialEq, Clone)]
struct ScoredUrl {
    url: String,
    score: u8,
}

impl ScoredUrl {
    fn new(url: &str, score: u8) -> Self {
        ScoredUrl {
            url: url.to_owned(),
            score,
        }
    }
}

/// Given an asset URL, return a score based on the packer it uses. The better the packer, the
/// higher the score.
fn get_pack_score(name: &str) -> usize {
    let ext = match get_file_extension(name) {
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
fn select_best_url<'a>(ui: &Ui, u1: &'a ScoredUrl, u2: &'a ScoredUrl) -> &'a ScoredUrl {
    match u1.score.cmp(&u2.score) {
        Ordering::Greater => return u1,
        Ordering::Less => return u2,
        Ordering::Equal => (),
    };

    // We know get_lname() is going to succeed here, because it has already been called before
    let n1 = get_lname(&u1.url).unwrap();
    let n2 = get_lname(&u2.url).unwrap();

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

//- extract_arch_os -------------------------------------------------
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
) -> Option<(ArchOs, u8)> {
    let mut score = 3;
    let arch = find_in_iter(ARCH_VEC.iter(), name).or_else(|| {
        score -= 1;
        default_arch
    })?;
    let os = find_in_iter(OS_VEC.iter(), name).or_else(|| {
        score -= 1;
        default_os
    })?;
    Some((ArchOs::new(arch, os), score))
}

//- Utils -----------------------------------------------------------
fn get_file_extension(name: &str) -> Option<&str> {
    name.rsplit_once('.').map(|x| x.1)
}

fn get_file_name(url: &str) -> Result<String> {
    let (_, name) = url
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;
    Ok(name.to_string())
}

fn get_lname(url: &str) -> Result<String> {
    Ok(get_file_name(url)?.to_ascii_lowercase())
}

fn is_supported_name(name: &str) -> bool {
    let ext = match get_file_extension(name) {
        Some(x) => x,
        None => return true,
    };
    if UNSUPPORTED_EXTS.contains(ext) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use clyde::test_file_utils::get_fixture_path;

    fn check_extract_arch_os(filename: &str, expected: Option<ArchOs>) {
        let result = extract_arch_os(filename, None, None).map(|x| x.0);
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
            if result != Some((expected, 3)) {
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
        check_extract_arch_os(
            "rclone-v1.61.1-osx-arm64.zip",
            Some(ArchOs::new(Arch::Aarch64, Os::MacOs)),
        );
        check_extract_arch_os("bar-3.14.tar.gz", None);
    }

    #[test]
    fn test_extract_arch_os_default_values() {
        let result = extract_arch_os("ninja-windows.zip", Some(Arch::X86_64), None);
        assert_eq!(result, Some((ArchOs::new(Arch::X86_64, Os::Windows), 2)));

        let result = extract_arch_os("ninja-mac.zip", Some(Arch::X86_64), None);
        assert_eq!(result, Some((ArchOs::new(Arch::X86_64, Os::MacOs), 2)));
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
            select_best_url(
                &ui,
                &ScoredUrl::new("https://example.com/foo.gz", 1),
                &ScoredUrl::new("https://example.com/foo", 1)
            ),
            &ScoredUrl::new("https://example.com/foo.gz", 1)
        );
        assert_eq!(
            select_best_url(
                &ui,
                &ScoredUrl::new("https://example.com/foo.gz", 1),
                &ScoredUrl::new("https://example.com/foo.xz", 1),
            ),
            &ScoredUrl::new("https://example.com/foo.xz", 1)
        );

        // libc is more important than compression
        assert_eq!(
            select_best_url(
                &ui,
                &ScoredUrl::new("https://example.com/foo-musl", 1),
                &ScoredUrl::new("https://example.com/foo-glibc.gz", 1),
            ),
            &ScoredUrl::new("https://example.com/foo-musl", 1)
        );
    }

    #[test]
    fn test_select_best_urls_applies_include() {
        let ui = Ui::default();

        let options = BestUrlOptions::new(None, None, Some(Regex::new("^foo-cli").unwrap()));

        let result = select_best_urls(
            &ui,
            &[
                "https://example.com/foo/foo-cli-x86_64-linux.gz".to_string(),
                "https://example.com/foo/foo-extra-x86_64-linux.gz".to_string(),
            ],
            options,
        )
        .unwrap();

        let expected = HashMap::from([(
            ArchOs::new(Arch::X86_64, Os::Linux),
            "https://example.com/foo/foo-cli-x86_64-linux.gz".to_string(),
        )]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_select_best_urls_prefers_not_using_default_arch_os() {
        let ui = Ui::default();

        // GIVEN a BestUrlOptions instance with a default arch
        let options = BestUrlOptions::new(Some(Arch::X86_64), None, None);

        // AND 2 URLs: URL1 uses an unknown arch, URL2 uses the default arch
        // WHEN calling select_best_urls() on [URL1, URL2]
        let result = select_best_urls(
            &ui,
            &[
                "https://example.com/foo/foo-z80-linux.gz".to_string(),
                "https://example.com/foo/foo-x86_64-linux.gz".to_string(),
            ],
            options,
        )
        .unwrap();

        // THEN it selects the URL with the known arch (URL2)
        let expected = HashMap::from([(
            ArchOs::new(Arch::X86_64, Os::Linux),
            "https://example.com/foo/foo-x86_64-linux.gz".to_string(),
        )]);
        assert_eq!(result, expected);
    }
}
