use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use semver::Version;

use clyde::arch_os::ArchOs;
use clyde::checksum::compute_checksum;
use clyde::file_cache::FileCache;
use clyde::package::{Build, Package};

type MatchingMap = HashMap<&'static str, Vec<&'static str>>;

lazy_static! {
    static ref ARCH_MAP: MatchingMap = {
        let mut map = HashMap::new();
        map.insert("x86_64", vec!["x86_64", "amd64"]);
        map.insert("x86", vec!["x86", "386"]);
        map.insert("aarch64", vec!["aarch64", "arm64"]);
        map
    };
    static ref OS_MAP: MatchingMap = {
        let mut map = HashMap::new();
        map.insert("linux", vec!["linux"]);
        map.insert("macos", vec!["macos", "apple", "darwin"]);
        map.insert("windows", vec!["windows", "win32"]);
        map
    };
}

fn compute_url_checksum(cache: &FileCache, url: &str) -> Result<String> {
    let path = cache.download(url)?;
    compute_checksum(&path)
}

// Must take an iterator as argument because each *_MAP is a unique type
fn find_in_iter(
    iter: Iter<'_, &'static str, Vec<&'static str>>,
    name: &str,
) -> Option<&'static str> {
    for (key, tokens) in iter {
        for token in tokens {
            if name.contains(token) {
                return Some(key);
            }
        }
    }
    None
}

fn extract_arch_os(name: &str) -> Option<ArchOs> {
    let arch = find_in_iter(ARCH_MAP.iter(), name)?;
    let os = find_in_iter(OS_MAP.iter(), name)?;
    Some(ArchOs::new(arch, os))
}

fn add_build(
    cache: &FileCache,
    release: &mut HashMap<ArchOs, Build>,
    arch_os: &ArchOs,
    url: &str,
) -> Result<()> {
    let checksum = compute_url_checksum(cache, url)?;

    let build = Build {
        url: url.to_string(),
        sha256: checksum,
    };

    release.insert(arch_os.clone(), build);

    Ok(())
}

pub fn add_builds(
    path: &Path,
    version: &str,
    arch_os: &Option<String>,
    urls: &Vec<String>,
) -> Result<()> {
    let cache = FileCache::new(&PathBuf::from("/tmp"));
    let package = Package::from_file(path)?;

    let version = Version::parse(version)?;
    let mut release = match package.releases.get(&version) {
        Some(x) => x.clone(),
        None => HashMap::<ArchOs, Build>::new(),
    };

    if let Some(arch_os) = arch_os {
        let url = urls.first().unwrap();
        let arch_os = ArchOs::parse(arch_os)?;
        add_build(&cache, &mut release, &arch_os, url)?;
    } else {
        for url in urls {
            let (_, name) = url
                .rsplit_once('/')
                .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;

            if let Some(arch_os) = extract_arch_os(&name.to_ascii_lowercase()) {
                if add_build(&cache, &mut release, &arch_os, url).is_err() {
                    eprintln!("Can't add {:?} build from {}", arch_os, url);
                }
            } else {
                eprintln!("Can't extract arch-os from {}", name);
            }
        }
    }

    let new_package = package.replace_release(&version, release);
    new_package.to_file(path)?;

    Ok(())
}
