use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use semver::Version;

use clyde::arch_os::ArchOs;
use clyde::checksum::compute_checksum;
use clyde::download::download;
use clyde::file_cache::FileCache;
use clyde::package::{Build, Package};

fn compute_url_checksum(cache: &FileCache, url: &str) -> Result<String> {
    let (_, name) = url
        .rsplit_once('/')
        .ok_or_else(|| anyhow!("Can't find archive name in URL {}", url))?;

    let path = cache.get_path(&OsString::from(name));

    download(url, &path)?;

    compute_checksum(&path)
}

pub fn add_build(path: &Path, version: &str, arch_os: &str, url: &str) -> Result<()> {
    let cache = FileCache::new(&PathBuf::from("/tmp"));
    let checksum = compute_url_checksum(&cache, url)?;

    let package = Package::from_file(path)?;
    let version = Version::parse(version)?;
    let arch_os = ArchOs::parse(arch_os)?;

    let mut release = match package.releases.get(&version) {
        Some(x) => x.clone(),
        None => HashMap::<ArchOs, Build>::new(),
    };

    let build = Build {
        url: url.to_string(),
        sha256: checksum,
    };

    release.insert(arch_os, build);

    let new_package = package.replace_release(&version, release);
    new_package.to_file(path)?;

    Ok(())
}
