use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::os::unix;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use sha2::{digest::DynDigest, Sha256};

use hex;

use crate::app::App;
use crate::package::Build;
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

fn unpack(archive: &Path, pkg_dir: &Path) -> Result<()> {
    println!("Unpacking...");
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(pkg_dir)?;
    Ok(())
}

fn install_binaries(
    pkg_dir: &Path,
    bin_dir: &Path,
    binaries: &HashMap<String, String>,
) -> Result<()> {
    println!("Installing binaries...");
    fs::create_dir_all(&bin_dir)?;
    for (src, dst) in binaries.iter() {
        let src_path = pkg_dir.join(src);
        let dst_path = bin_dir.join(dst);
        unix::fs::symlink(src_path, dst_path)?;
    }
    Ok(())
}

pub fn install(app: &App, package_name: &str) -> Result<()> {
    let package = app.store.get_package(package_name)?;
    println!("Installing {}", package_name);

    let build: &Build = package
        .get_latest_build()
        .ok_or_else(|| anyhow!("No build available for {}", package_name))?;

    let dst_name = build.get_archive_name()?;
    let dst_path = app.download_cache.get_path(&dst_name);

    if dst_path.exists() {
        println!("Already downloaded");
    } else {
        download(&build.url, &dst_path)?;
    }

    verify_checksum(&dst_path, &build.sha256)?;

    let pkg_dir = app.pkg_base_dir.join(&package.name);
    if pkg_dir.exists() {
        fs::remove_dir_all(&pkg_dir)?
    }
    unpack(&dst_path, &pkg_dir)?;

    install_binaries(&pkg_dir, &app.bin_dir, &build.binaries)?;

    Ok(())
}
