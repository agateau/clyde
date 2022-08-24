// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use semver::VersionReq;

use crate::app::App;
use crate::arch_os::ArchOs;
use crate::checksum::verify_checksum;
use crate::ui::Ui;
use crate::uninstall::uninstall_package;
use crate::unpacker::get_unpacker;
use crate::vars::{expand_vars, VarsMap};

fn unpack(archive: &Path, pkg_dir: &Path, strip: u32) -> Result<Option<String>> {
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(pkg_dir, strip)
}

/// Create dir containing `path` and all its parents, if necessary
fn create_parent_dir(path: &Path) -> Result<()> {
    let dir = path.parent().unwrap();
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create directory {:?}", &dir))?;
    Ok(())
}

/// Install `src_path` in `install_dir/dst`. If `src_path` is a dir, install its content
/// recursively.
/// Add the installed files to `installed_files`.
fn install_file_entry(
    installed_files: &mut HashSet<PathBuf>,
    src_path: &Path,
    install_dir: &Path,
    dst: &Path,
    vars: &HashMap<String, String>,
) -> Result<()> {
    if src_path.is_dir() {
        for entry in fs::read_dir(src_path)? {
            let entry = entry?;
            let sub_src_path = entry.path();
            let dst_name = sub_src_path
                .file_name()
                .ok_or_else(|| anyhow!("{:?} has no file name!", sub_src_path))?;
            let sub_dst = dst.join(dst_name);
            install_file_entry(installed_files, &sub_src_path, install_dir, &sub_dst, vars)?;
        }
    } else {
        // rel_dst_path is the destination path, relative to install_dir
        let rel_dst_path = if dst.to_str().unwrap().ends_with('/') {
            // dst is a dir, turn it into a file
            let file_name = src_path
                .file_name()
                .ok_or_else(|| anyhow!("{:?} has no file name!", src_path))?;
            dst.join(file_name)
        } else {
            dst.to_path_buf()
        };

        let dst_path = install_dir.join(&rel_dst_path);

        create_parent_dir(&dst_path)?;

        fs::rename(&src_path, &dst_path)
            .with_context(|| format!("Failed to move {:?} to {:?}", &src_path, &dst_path))?;

        installed_files.insert(rel_dst_path);
    }
    Ok(())
}

/// Install all files from a `${arch_os}.files` mapping.
/// Returns a set of the installed files.
fn install_files(
    pkg_dir: &Path,
    install_dir: &Path,
    file_map: &BTreeMap<String, String>,
    vars: &VarsMap,
) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::<PathBuf>::new();

    fs::create_dir_all(&install_dir)?;
    for (src, dst) in file_map.iter() {
        let src = expand_vars(src, vars)?;
        let dst = expand_vars(dst, vars)?;

        let src_path = pkg_dir.join(src);
        install_file_entry(&mut files, &src_path, install_dir, Path::new(&dst), vars)?;
    }
    Ok(files)
}

fn parse_package_name_arg(arg: &str) -> Result<(&str, VersionReq)> {
    let split = arg.split_once('@');
    match split {
        None => Ok((arg, VersionReq::STAR)),
        Some((name, requested_str)) => {
            let version = VersionReq::parse(requested_str).with_context(|| {
                format!("Failed to parse requested version ('{requested_str}')")
            })?;
            Ok((name, version))
        }
    }
}

fn create_vars_map(asset_name: &Option<String>, package_name: &str) -> VarsMap {
    let mut map = VarsMap::new();

    map.insert(
        "exe_ext".into(),
        if cfg!(windows) {
            ".exe".into()
        } else {
            "".into()
        },
    );

    map.insert("doc_dir".into(), format!("share/doc/{}/", package_name));
    if let Some(asset_name) = asset_name {
        map.insert("asset_name".into(), asset_name.clone());
    }

    map
}

pub fn install(app: &App, ui: &Ui, package_name_args: &Vec<String>) -> Result<()> {
    for package_name_arg in package_name_args {
        let (package_name, requested_version) = parse_package_name_arg(package_name_arg)?;
        install_with_package_and_requested_version(app, ui, package_name, &requested_version)?;
    }
    Ok(())
}

pub fn install_with_package_and_requested_version(
    app: &App,
    ui: &Ui,
    package_name: &str,
    requested_version: &VersionReq,
) -> Result<()> {
    let db = &app.database;

    let arch_os = ArchOs::current();

    let package = app.store.get_package(package_name)?;

    let version = package
        .get_version_matching(requested_version)
        .ok_or_else(|| {
            anyhow!("No version matching '{requested_version}' available for {package_name}")
        })?;

    let build = package
        .get_asset(version, &arch_os)
        .ok_or_else(|| anyhow!("No {arch_os} asset available for {package_name} {version}"))?;

    let install = package
        .get_install(version, &arch_os)
        .ok_or_else(|| anyhow!("No files instruction for {}", package_name))?;

    let installed_version = db.get_package_version(package_name)?;
    if installed_version == Some(version.clone()) {
        return Err(anyhow!("{} {} is already installed", package_name, version));
    }
    ui.info(&format!("Installing {} {}", package_name, version));

    let ui = ui.nest();
    let asset_path = app.download_cache.download(&ui, &build.url)?;

    ui.info("Verifying asset integrity");
    verify_checksum(&asset_path, &build.sha256)?;

    let unpack_dir = app.tmp_dir.join(&package.name);
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?
    }

    ui.info("Unpacking asset");
    let asset_name = unpack(&asset_path, &unpack_dir, install.strip)?;

    if installed_version.is_some() {
        // A different version is already installed, uninstall it first
        uninstall_package(app, &ui, package_name)?;
    }

    ui.info("Installing files");
    let installed_files = install_files(
        &unpack_dir,
        &app.install_dir,
        &install.files,
        &create_vars_map(&asset_name, package_name),
    )?;
    db.add_package(&package.name, version, requested_version, &installed_files)?;

    ui.info("Cleaning");
    fs::remove_dir_all(&unpack_dir)
        .with_context(|| format!("Failed to delete {}", unpack_dir.display()))?;
    fs::remove_file(&asset_path)
        .with_context(|| format!("Failed to delete {}", asset_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::test_file_utils::{create_tree, list_tree, pathbufset_from_strings};

    #[test]
    fn test_parse_package_name_arg() {
        assert_eq!(
            parse_package_name_arg("foo").unwrap(),
            ("foo", VersionReq::STAR)
        );
        assert_eq!(
            parse_package_name_arg("foo@1.2").unwrap(),
            ("foo", VersionReq::parse("1.2").unwrap())
        );
        assert_eq!(
            parse_package_name_arg("foo@1.*").unwrap(),
            ("foo", VersionReq::parse("1.*").unwrap())
        );
    }

    #[test]
    fn install_files_should_copy_files() {
        let dir = assert_fs::TempDir::new().unwrap();
        let pkg_dir = dir.join("pkg");
        let inst_dir = dir.join("inst");
        create_tree(&pkg_dir, &["bin/foo-1.2", "README.md"]);

        let files: BTreeMap<String, String> = BTreeMap::from([
            ("bin/foo-1.2".to_string(), "bin/foo".to_string()),
            ("README.md".to_string(), "share/doc/foo/".to_string()),
        ]);

        let result = install_files(&pkg_dir, &inst_dir, &files, &HashMap::new());
        assert_eq!(
            result.unwrap(),
            pathbufset_from_strings(&["bin/foo", "share/doc/foo/README.md"])
        );
        assert_eq!(
            list_tree(&inst_dir).unwrap(),
            pathbufset_from_strings(&["bin/foo", "share/doc/foo/README.md"])
        );
    }

    #[test]
    fn install_files_should_expand_vars() {
        let dir = assert_fs::TempDir::new().unwrap();
        let pkg_dir = dir.join("pkg");
        let inst_dir = dir.join("inst");
        create_tree(&pkg_dir, &["bin/foo.exe", "README.md"]);

        let files: BTreeMap<String, String> = BTreeMap::from([
            ("bin/foo${exe_ext}".to_string(), "bin/".to_string()),
            ("README.md".to_string(), "${doc_dir}".to_string()),
        ]);

        let vars = HashMap::from([
            ("exe_ext".to_string(), ".exe".to_string()),
            ("doc_dir".to_string(), "share/doc/foo/".to_string()),
        ]);
        let result = install_files(&pkg_dir, &inst_dir, &files, &vars);
        assert_eq!(
            result.unwrap(),
            pathbufset_from_strings(&["bin/foo.exe", "share/doc/foo/README.md"])
        );
        assert_eq!(
            list_tree(&inst_dir).unwrap(),
            pathbufset_from_strings(&["bin/foo.exe", "share/doc/foo/README.md"])
        );
    }

    #[test]
    fn install_files_should_merge_dirs() {
        // GIVEN a prefix with a `share/man/f1` file
        let dir = assert_fs::TempDir::new().unwrap();
        let inst_dir = dir.join("inst");
        create_tree(&inst_dir, &["share/man/f1"]);

        // AND a package containing a `share/man/f2` file and installing `share` in `share`
        let pkg_dir = dir.join("pkg");
        create_tree(&pkg_dir, &["share/man/f2"]);

        let files: BTreeMap<String, String> =
            BTreeMap::from([("share".to_string(), "share".to_string())]);

        // WHEN install_files() is called
        let result = install_files(&pkg_dir, &inst_dir, &files, &HashMap::new());

        // THEN the prefix contain both files
        assert_eq!(
            list_tree(&inst_dir).unwrap(),
            pathbufset_from_strings(&["share/man/f1", "share/man/f2"])
        );

        // AND install_files() returns the path to `share/man/f2`
        assert_eq!(
            result.unwrap(),
            HashSet::from([PathBuf::from("share/man/f2")])
        );
    }
}
