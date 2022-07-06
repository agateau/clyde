use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use clyde::checksum::compute_checksum;
use clyde::file_cache::FileCache;
use clyde::package::{Build, Install, InternalPackage};
use clyde::vars::expand_var;

use serde_json::Value;

struct Arch<'a> {
    pub clyde: &'a str,
    pub hermit: &'a str,
}

struct Os<'a> {
    pub clyde: &'a str,
    pub hermit: &'a str,
}

const SUPPORTED_ARCHS: &[Arch] = &[
    Arch {
        clyde: "x86_64",
        hermit: "amd64",
    },
    Arch {
        clyde: "aarch64",
        hermit: "arm64",
    },
];

const SUPPORTED_OSES: &[Os] = &[
    Os {
        clyde: "linux",
        hermit: "linux",
    },
    Os {
        clyde: "macos",
        hermit: "darwin",
    },
];

fn compute_url_checksum(cache: &FileCache, url: &str) -> Result<String> {
    let path = cache.download(url)?;
    compute_checksum(&path)
}

fn read_versions(version_value: &Value) -> Vec<String> {
    let mut versions = Vec::<String>::new();
    let obj = version_value.as_object().unwrap();
    for (key, value) in obj.iter() {
        if let Some(ch) = key.chars().next() {
            if ch.is_numeric() {
                versions.push(key.to_string());
                versions.extend(read_versions(value));
            }
        }
    }
    versions
}

/// Return a map of arch_os => url_templace
fn read_archive_templates(value: &Value) -> HashMap<String, String> {
    let mut map = HashMap::<String, String>::new();
    for os in SUPPORTED_OSES {
        if let Some(entry) = value[&os.hermit].as_object() {
            let source = entry["source"].as_str().unwrap();
            let source = expand_var(source, "os", os.hermit);
            for arch in SUPPORTED_ARCHS {
                let source = expand_var(&source, "arch", arch.hermit);
                let source = expand_var(&source, "xarch", arch.clyde);
                let arch_os = format!("{}-{}", arch.clyde, os.clyde);
                map.insert(arch_os, source);
            }
        }
    }
    map
}

fn create_releases(
    versions: &[String],
    archive_templates: &HashMap<String, String>,
) -> BTreeMap<String, BTreeMap<String, Build>> {
    let file_cache = FileCache::new(&PathBuf::from("/tmp"));

    // version => (ArchOs => Build)
    let mut map = BTreeMap::<String, BTreeMap<String, Build>>::new();
    for version in versions {
        let mut build_map = BTreeMap::<String, Build>::new();
        for (arch_os, template) in archive_templates.iter() {
            let url = expand_var(template, "version", version);
            if let Ok(sha256) = compute_url_checksum(&file_cache, &url) {
                let build = Build { url, sha256 };
                build_map.insert(arch_os.to_string(), build);
            }
        }
        map.insert(version.clone(), build_map);
    }
    map
}

fn create_installs(version: &str, value: &Value) -> BTreeMap<String, BTreeMap<String, Install>> {
    let strip: u32 = value["strip"].as_u64().unwrap_or(0).try_into().unwrap();
    let mut files = BTreeMap::<String, String>::new();

    for binary_value in value["binaries"].as_array().unwrap() {
        let binary = binary_value.as_str().unwrap();
        files.insert(binary.to_string(), format!("bin/{}", binary));
    }
    let install = Install { strip, files };

    let mut installs = BTreeMap::<String, BTreeMap<String, Install>>::new();
    let mut install_for_arch_os_map = BTreeMap::<String, Install>::new();
    install_for_arch_os_map.insert("any".to_string(), install);
    installs.insert(version.to_string(), install_for_arch_os_map);
    installs
}

pub fn import_hermit(package_file: &str) -> Result<()> {
    let path = PathBuf::from(package_file);

    // Get name
    let name = path
        .file_stem()
        .ok_or_else(|| anyhow!("Can't get name from {:?}", path))?
        .to_str()
        .unwrap();

    // Parse HCL
    let input = fs::read_to_string(&path)?;
    let value: Value =
        hcl::from_str(&input).map_err(|x| anyhow!("Failed to parse {:?}:\n{}", path, x))?;

    let versions = read_versions(&value["version"]);

    // Return a (String(ArchOs) => String(template))
    let archive_templates = read_archive_templates(&value);

    let releases = create_releases(&versions, &archive_templates);

    let first_version = &versions[0];
    let installs = create_installs(first_version, &value);

    let description = value["description"]
        .as_str()
        .ok_or_else(|| anyhow!("Failed to read description field"))?
        .trim_matches('"')
        .to_string();

    let homepage = value["homepage"]
        .as_str()
        .unwrap_or("")
        .trim_matches('"')
        .to_string();

    let pkg = InternalPackage {
        name: name.to_string(),
        description,
        homepage,
        releases: Some(releases),
        installs: Some(installs),
    };

    let out = serde_yaml::to_string(&pkg).unwrap();

    println!("{}", out);

    Ok(())
}
