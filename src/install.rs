use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use sha2::{digest::DynDigest, Sha256};

use hex;

use crate::app::App;
use crate::arch_os::ArchOs;
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

fn compute_checksum(path: &Path) -> Result<Box<[u8]>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::default();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize_reset())
}

fn verify_checksum(path: &Path, expected: &str) -> Result<()> {
    println!("Verifying checksum...");
    let result = compute_checksum(path)?;

    let actual = hex::encode(result);

    if actual != expected {
        return Err(anyhow!(
            "Checksums do not match.\nExpected: {}\nReceived: {}",
            expected,
            actual
        ));
    }
    Ok(())
}

fn unpack(archive: &Path, pkg_dir: &Path, strip: u32) -> Result<()> {
    println!("Unpacking...");
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(pkg_dir, strip)?;
    Ok(())
}

fn install_file(src_path: &Path, dst_path: &Path) -> Result<()> {
    if src_path.is_dir() {
        for entry in fs::read_dir(src_path)? {
            let entry = entry?;
            let sub_src_path = entry.path();
            let dst_name = sub_src_path
                .file_name()
                .ok_or_else(|| anyhow!("{:?} has no file name!", sub_src_path))?;
            let sub_dst_path = dst_path.join(dst_name);
            install_file(&sub_src_path, &sub_dst_path)?;
        }
    } else {
        let dst_dir = dst_path.parent().unwrap();
        fs::create_dir_all(&dst_dir)
            .map_err(|err| anyhow!("Failed to create directory {:?}: {}", &dst_dir, err))?;

        fs::rename(src_path, dst_path)
            .map_err(|err| anyhow!("Failed to move {:?} to {:?}: {}", &src_path, &dst_path, err))?;
    }
    Ok(())
}

fn install_files(
    pkg_dir: &Path,
    install_dir: &Path,
    files: &HashMap<String, String>,
) -> Result<()> {
    println!("Installing files...");
    fs::create_dir_all(&install_dir)?;
    for (src, dst) in files.iter() {
        let src_path = pkg_dir.join(src);
        if !src_path.exists() {
            return Err(anyhow!("Source file {:?} does not exist", src_path));
        }

        let dst_path = install_dir.join(dst);

        install_file(&src_path, &dst_path)?;
    }
    Ok(())
}

pub fn install(app: &App, package_name: &str) -> Result<()> {
    let arch_os = ArchOs::current();

    let package = app.store.get_package(package_name)?;
    println!("Installing {}", package_name);

    let version = package
        .get_latest_version()
        .ok_or_else(|| anyhow!("No build available for {}", package_name))?;

    let build = package
        .get_build(version, &arch_os)
        .ok_or_else(|| anyhow!("No build available for {}", package_name))?;

    let install = package
        .get_install(version, &arch_os)
        .ok_or_else(|| anyhow!("No files instruction for {}", package_name))?;

    let archive_name = build.get_archive_name()?;
    let archive_path = app.download_cache.get_path(&archive_name);

    if archive_path.exists() {
        println!("Already downloaded");
    } else {
        download(&build.url, &archive_path)?;
    }

    verify_checksum(&archive_path, &build.sha256)?;

    let unpack_dir = app.tmp_dir.join(&package.name);
    if unpack_dir.exists() {
        fs::remove_dir_all(&unpack_dir)?
    }
    unpack(&archive_path, &unpack_dir, install.strip)?;

    install_files(&unpack_dir, &app.install_dir, &install.files)?;

    fs::remove_dir_all(&unpack_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::File;
    use std::path::{Path, PathBuf};
    use std::vec::Vec;

    fn create_tree(root: &Path, files: &[&str]) {
        for file in files {
            let path = root.join(file);
            fs::create_dir_all(&path.parent().unwrap()).unwrap();
            File::create(&path).unwrap();
        }
    }

    fn list_tree_internal(root: &Path, parent: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::<PathBuf>::new();
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(list_tree_internal(&root, &path)?);
            } else {
                let rel_path = path.strip_prefix(&root)?;
                files.push(rel_path.to_path_buf());
            }
        }
        Ok(files)
    }

    fn list_tree(root: &Path) -> Result<Vec<String>> {
        let file_vec = list_tree_internal(&root, &root)?;
        let file_array = file_vec
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();
        Ok(file_array)
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
        assert!(result.is_ok());
        assert!(list_tree(&inst_dir).unwrap() == vec!["bin/foo", "share/doc/foo/README.md"]);
    }

    #[test]
    fn install_files_should_merge_dirs() {
        let dir = assert_fs::TempDir::new().unwrap();
        let pkg_dir = dir.join("pkg");
        create_tree(&pkg_dir, &["share/man/f2"]);

        let inst_dir = dir.join("inst");
        create_tree(&inst_dir, &["share/man/f1"]);

        let files: HashMap<String, String> =
            HashMap::from([("share".to_string(), "share".to_string())]);

        let result = install_files(&pkg_dir, &inst_dir, &files);
        assert!(result.is_ok(), "{:?}", result);
        assert!(list_tree(&inst_dir).unwrap() == vec!["share/man/f1", "share/man/f2"]);
    }
}
