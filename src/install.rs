use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use sha2::{digest::DynDigest, Sha256};

use hex;

use crate::app::App;
use crate::package::Release;
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

fn unpack(archive: &Path, binaries: &HashMap<String, String>, bin_dir: &Path) -> Result<()> {
    let unpacker = get_unpacker(archive)?;
    unpacker.unpack(binaries, bin_dir)?;
    Ok(())
}

pub fn install(app: &App, package_name: &str) -> Result<()> {
    let package = app.store.get_package(package_name)?;
    println!("Installing {}", package_name);

    let release: &Release = package
        .get_latest_release()
        .ok_or_else(|| anyhow!("No release available for {}", package_name))?;

    let dst_name = release.get_archive_name()?;
    let dst_path = app.download_cache.get_path(&dst_name);

    if dst_path.exists() {
        println!("Already downloaded");
    } else {
        download(&release.url, &dst_path)?;
    }

    verify_checksum(&dst_path, &release.sha256)?;

    unpack(&dst_path, &release.binaries, &app.bin_dir)?;

    Ok(())
}
