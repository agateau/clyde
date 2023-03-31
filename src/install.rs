// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{BTreeMap, HashSet};
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

#[derive(Debug, PartialEq)]
/// Store the details of an install, used by install_package()
/// and install_packages()
pub struct InstallRequest {
    pub name: String,
    pub version: VersionReq,
}

impl InstallRequest {
    pub fn new(package_name: &str, version: VersionReq) -> Self {
        InstallRequest {
            name: package_name.into(),
            version,
        }
    }
}

fn unpack(archive: &Path, pkg_dir: &Path, strip: u32) -> Result<Option<String>> {
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(pkg_dir, strip)
}

/// Create dir containing `path` and all its parents, if necessary
fn create_parent_dir(path: &Path) -> Result<()> {
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir).with_context(|| format!("Failed to create directory {:?}", &dir))?;
    Ok(())
}

#[derive(Clone, Copy)]
enum InstallMode {
    Move,
    Copy,
}

/// Install `src_path` in `install_dir/dst`. If `src_path` is a dir, install its content
/// recursively.
/// Add the installed files to `installed_files`.
fn install_file_entry(
    install_mode: InstallMode,
    installed_files: &mut HashSet<PathBuf>,
    src_path: &Path,
    install_dir: &Path,
    dst: &Path,
) -> Result<()> {
    if src_path.is_dir() {
        for entry in fs::read_dir(src_path)? {
            let entry = entry?;
            let sub_src_path = entry.path();
            let dst_name = sub_src_path
                .file_name()
                .ok_or_else(|| anyhow!("{:?} has no file name!", sub_src_path))?;
            let sub_dst = dst.join(dst_name);
            install_file_entry(
                install_mode,
                installed_files,
                &sub_src_path,
                install_dir,
                &sub_dst,
            )?;
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

        match install_mode {
            InstallMode::Move => {
                fs::rename(src_path, &dst_path).with_context(|| {
                    format!("Failed to move {:?} to {:?}", &src_path, &dst_path)
                })?;
            }
            InstallMode::Copy => {
                fs::copy(src_path, &dst_path).with_context(|| {
                    format!("Failed to copy {:?} to {:?}", &src_path, &dst_path)
                })?;
            }
        }

        installed_files.insert(rel_dst_path);
    }
    Ok(())
}

/// Install all files from a `${arch_os}.files` mapping.
/// Add the installed files to `installed_files`.
fn install_files(
    install_mode: InstallMode,
    installed_files: &mut HashSet<PathBuf>,
    pkg_dir: &Path,
    install_dir: &Path,
    file_map: &BTreeMap<String, String>,
    vars: &VarsMap,
) -> Result<()> {
    fs::create_dir_all(install_dir)?;
    for (src, dst) in file_map.iter() {
        let src = expand_vars(src, vars)?;
        let dst = if dst.is_empty() {
            // If dst is empty it means the destination is the same as the source
            src.clone()
        } else {
            expand_vars(dst, vars)?
        };

        let src_path = pkg_dir.join(src);
        install_file_entry(
            install_mode,
            installed_files,
            &src_path,
            install_dir,
            Path::new(&dst),
        )?;
    }
    Ok(())
}

fn parse_package_name_arg(arg: &str) -> Result<InstallRequest> {
    let split = arg.split_once('@');
    match split {
        None => Ok(InstallRequest::new(arg, VersionReq::STAR)),
        Some((name, requested_str)) => {
            let version = VersionReq::parse(requested_str).with_context(|| {
                format!("Failed to parse requested version ('{requested_str}') from '{arg}'")
            })?;
            Ok(InstallRequest::new(name, version))
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

    map.insert("doc_dir".into(), format!("share/doc/{package_name}/"));
    map.insert(
        "bash_comp_dir".into(),
        // The extra "/completions/" is required by bash-completions
        "share/bash-completions/completions/".into(),
    );
    map.insert("zsh_comp_dir".into(), "share/zsh-completions/".into());
    if let Some(asset_name) = asset_name {
        map.insert("asset_name".into(), asset_name.clone());
    }

    map
}

pub fn install_cmd(
    app: &App,
    ui: &Ui,
    reinstall: bool,
    package_name_args: &[String],
) -> Result<()> {
    let install_requests = package_name_args
        .iter()
        .map(|name| parse_package_name_arg(name))
        .collect::<Result<Vec<InstallRequest>>>()?;
    install_packages(app, ui, reinstall, &install_requests)
}

pub fn install_packages(
    app: &App,
    ui: &Ui,
    reinstall: bool,
    install_requests: &Vec<InstallRequest>,
) -> Result<()> {
    for request in install_requests {
        install_package(app, ui, reinstall, request)?;
    }
    Ok(())
}

pub fn install_package(
    app: &App,
    ui: &Ui,
    reinstall: bool,
    install_request: &InstallRequest,
) -> Result<()> {
    let db = &app.database;

    let arch_os = ArchOs::current();

    let package = app.store.get_package(&install_request.name)?;

    let version = package
        .get_version_matching(&install_request.version)
        .ok_or_else(|| {
            anyhow!(
                "No version matching '{}' available for {}",
                &install_request.version,
                &package.name
            )
        })?;

    let build = package.get_asset(version, &arch_os).ok_or_else(|| {
        anyhow!(
            "No {arch_os} asset available for {} {version}",
            &package.name
        )
    })?;

    let install = package
        .get_install(version, &arch_os)
        .ok_or_else(|| anyhow!("No files instruction for {}", &package.name))?;

    let installed_version = db.get_package_version(&package.name)?;
    if !reinstall && installed_version == Some(version.clone()) {
        return Err(anyhow!(
            "{} {} is already installed",
            &package.name,
            version
        ));
    }
    ui.info(&format!("Installing {} {}", &package.name, &version));

    let ui = ui.nest();
    let asset_path = app
        .download_cache
        .download(&ui, &package.name, version, &build.url)?;

    ui.info("Verifying asset integrity");
    match verify_checksum(&asset_path, &build.sha256) {
        Ok(()) => {}
        Err(err) => {
            fs::remove_file(&asset_path)?;
            return Err(err);
        }
    };

    let unpack_dir = app.tmp_dir.join(&package.name);
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?
    }

    ui.info("Unpacking asset");
    let asset_name = unpack(&asset_path, &unpack_dir, install.strip)?;

    if installed_version.is_some() {
        // The package is already installed: either it's a different version, or we were called
        // with --reinstall. Uninstall it first.
        uninstall_package(app, &ui, &package.name)?;
    }

    ui.info("Installing files");
    let map = create_vars_map(&asset_name, &package.name);
    let mut installed_files = HashSet::<PathBuf>::new();
    install_files(
        InstallMode::Move,
        &mut installed_files,
        &unpack_dir,
        &app.install_dir,
        &install.files,
        &map,
    )?;
    if package.extra_files_dir.exists() {
        ui.info("Installing extra files");
        install_files(
            InstallMode::Copy,
            &mut installed_files,
            &package.extra_files_dir,
            &app.install_dir,
            &install.extra_files,
            &map,
        )?;
    }
    db.add_package(
        &package.name,
        version,
        &install_request.version,
        &installed_files,
    )?;

    ui.info("Cleaning");
    fs::remove_dir_all(&unpack_dir)
        .with_context(|| format!("Failed to delete {}", unpack_dir.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::test_file_utils::{create_tree, list_tree, pathbufset_from_strings};

    #[test]
    fn test_parse_package_name_arg() {
        assert_eq!(
            parse_package_name_arg("foo").unwrap(),
            InstallRequest::new("foo", VersionReq::STAR)
        );
        assert_eq!(
            parse_package_name_arg("foo@1.2").unwrap(),
            InstallRequest::new("foo", VersionReq::parse("1.2").unwrap())
        );
        assert_eq!(
            parse_package_name_arg("foo@1.*").unwrap(),
            InstallRequest::new("foo", VersionReq::parse("1.*").unwrap())
        );
    }

    #[test]
    fn install_files_should_copy_files() {
        // GIVEN an unpacked package with files:
        // bin/foo-1.2
        // bin/food
        // README.md
        let dir = assert_fs::TempDir::new().unwrap();
        let pkg_dir = dir.join("pkg");
        let inst_dir = dir.join("inst");
        create_tree(&pkg_dir, &["bin/foo-1.2", "bin/food", "README.md"]);

        // And a map to install:
        // bin/foo-1.2 as bin/foo
        // bin/food as bin/food
        // README.md as share/doc/foo/README.md
        let files: BTreeMap<String, String> = BTreeMap::from([
            ("bin/foo-1.2".to_string(), "bin/foo".to_string()),
            ("bin/food".to_string(), "".to_string()),
            ("README.md".to_string(), "share/doc/foo/".to_string()),
        ]);

        // WHEN install_files() is called
        let mut installed_files = HashSet::<PathBuf>::new();
        let result = install_files(
            InstallMode::Move,
            &mut installed_files,
            &pkg_dir,
            &inst_dir,
            &files,
            &HashMap::new(),
        );

        // THEN it is OK
        assert!(result.is_ok());

        // AND installed_files contains the correct file list
        assert_eq!(
            installed_files,
            pathbufset_from_strings(&["bin/foo", "bin/food", "share/doc/foo/README.md"])
        );

        // AND the install dir contains the correct files
        assert_eq!(
            list_tree(&inst_dir).unwrap(),
            pathbufset_from_strings(&["bin/foo", "bin/food", "share/doc/foo/README.md"])
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

        let mut installed_files = HashSet::<PathBuf>::new();
        let result = install_files(
            InstallMode::Move,
            &mut installed_files,
            &pkg_dir,
            &inst_dir,
            &files,
            &vars,
        );

        assert!(result.is_ok());
        assert_eq!(
            installed_files,
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
        let mut installed_files = HashSet::<PathBuf>::new();
        let result = install_files(
            InstallMode::Move,
            &mut installed_files,
            &pkg_dir,
            &inst_dir,
            &files,
            &HashMap::new(),
        );

        // THEN the result is OK
        assert!(result.is_ok());

        // AND the prefix contain both files
        assert_eq!(
            list_tree(&inst_dir).unwrap(),
            pathbufset_from_strings(&["share/man/f1", "share/man/f2"])
        );

        // AND install_files() returns the path to `share/man/f2`
        assert_eq!(
            installed_files,
            HashSet::from([PathBuf::from("share/man/f2")])
        );
    }
}
