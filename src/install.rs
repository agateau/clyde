use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use semver::VersionReq;

use crate::app::App;
use crate::arch_os::ArchOs;
use crate::checksum::verify_checksum;
use crate::remove::remove;
use crate::unpacker::get_unpacker;

fn download(url: &str, dst_path: &Path) -> Result<()> {
    println!("Downloading {} to {:?}", url, dst_path);

    let mut cmd = Command::new("curl");
    cmd.args(["-L", "-o"]);
    cmd.arg(dst_path.as_os_str());
    cmd.arg(url);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("Download failed"));
    }
    Ok(())
}

fn unpack(archive: &Path, pkg_dir: &Path, strip: u32) -> Result<()> {
    println!("Unpacking...");
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(pkg_dir, strip)?;
    Ok(())
}

fn install_file(
    files: &mut HashSet<PathBuf>,
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
            install_file(files, &sub_src_path, install_dir, &sub_dst)?;
        }
    } else {
        let dst_path = install_dir.join(dst);
        let dst_dir = dst_path.parent().unwrap();
        fs::create_dir_all(&dst_dir)
            .map_err(|err| anyhow!("Failed to create directory {:?}: {}", &dst_dir, err))?;

        fs::rename(src_path, &dst_path)
            .map_err(|err| anyhow!("Failed to move {:?} to {:?}: {}", &src_path, &dst_path, err))?;

        files.insert(dst.to_path_buf());
    }
    Ok(())
}

fn install_files(
    pkg_dir: &Path,
    install_dir: &Path,
    file_map: &HashMap<String, String>,
) -> Result<HashSet<PathBuf>> {
    println!("Installing files...");
    let mut files = HashSet::<PathBuf>::new();

    fs::create_dir_all(&install_dir)?;
    for (src, dst) in file_map.iter() {
        let src_path = pkg_dir.join(src);
        if !src_path.exists() {
            return Err(anyhow!("Source file {:?} does not exist", src_path));
        }

        install_file(&mut files, &src_path, install_dir, &PathBuf::from(dst))?;
    }
    Ok(files)
}

fn parse_package_name_arg(arg: &str) -> Result<(&str, VersionReq)> {
    let split = arg.split_once('@');
    match split {
        None => Ok((arg, VersionReq::STAR)),
        Some((name, requested_str)) => {
            let version = VersionReq::parse(requested_str)?;
            Ok((name, version))
        }
    }
}

pub fn install(app: &App, package_name_arg: &str) -> Result<()> {
    let db = &app.database;

    let arch_os = ArchOs::current();

    let (package_name, requested_version) = parse_package_name_arg(package_name_arg)?;

    let package = app.store.get_package(package_name)?;

    let version = package
        .get_version_matching(&requested_version)
        .ok_or_else(|| anyhow!("No build available for {}", package_name))?;

    let build = package
        .get_build(version, &arch_os)
        .ok_or_else(|| anyhow!("No build available for {}", package_name))?;

    let install = package
        .get_install(version, &arch_os)
        .ok_or_else(|| anyhow!("No files instruction for {}", package_name))?;

    if let Some(installed_version) = db.get_package_version(package_name)? {
        if &installed_version == version {
            return Err(anyhow!("{} {} is already installed", package_name, version));
        }
        // A different version is already installed, remove it first
        remove(app, package_name)?;
    }
    println!("Installing {} {}...", package_name, version);

    let archive_name = build.get_archive_name()?;
    let archive_path = app.download_cache.get_path(&archive_name);

    if archive_path.exists() {
        println!("Already downloaded");
    } else {
        download(&build.url, &archive_path)?;
    }

    println!("Verifying checksum...");
    verify_checksum(&archive_path, &build.sha256)?;

    let unpack_dir = app.tmp_dir.join(&package.name);
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?
    }
    unpack(&archive_path, &unpack_dir, install.strip)?;

    let installed_files = install_files(&unpack_dir, &app.install_dir, &install.files)?;
    db.add_package(&package.name, version, &requested_version, &installed_files)?;

    fs::remove_dir_all(&unpack_dir)?;

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

        let files: HashMap<String, String> = HashMap::from([
            ("bin/foo-1.2".to_string(), "bin/foo".to_string()),
            (
                "README.md".to_string(),
                "share/doc/foo/README.md".to_string(),
            ),
        ]);

        let result = install_files(&pkg_dir, &inst_dir, &files);
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
    fn install_files_should_merge_dirs() {
        // GIVEN a prefix with a `share/man/f1` file
        let dir = assert_fs::TempDir::new().unwrap();
        let inst_dir = dir.join("inst");
        create_tree(&inst_dir, &["share/man/f1"]);

        // AND a package containing a `share/man/f2` file and installing `share` in `share`
        let pkg_dir = dir.join("pkg");
        create_tree(&pkg_dir, &["share/man/f2"]);

        let files: HashMap<String, String> =
            HashMap::from([("share".to_string(), "share".to_string())]);

        // WHEN install_files() is called
        let result = install_files(&pkg_dir, &inst_dir, &files);

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
