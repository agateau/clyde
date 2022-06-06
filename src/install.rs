use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, anyhow};

use crate::app::App;
use crate::package::{Package, Release};
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

fn unpack(archive: &Path, binaries: &HashMap<String, String>, bin_dir: &Path) -> Result<()> {
    let unpacker = get_unpacker(&archive)?;
    unpacker.unpack(&binaries, &bin_dir)?;
    Ok(())
}

pub fn install(app: &App, package_name: &str) -> Result<()> {
    println!("Installing {}", package_name);
    let package_def_path = PathBuf::from(package_name);
    let package = Package::from_file(&package_def_path)?;

    let release : &Release = package.releases.get(0).ok_or(
        anyhow!("No release in package")
    )?;

    let dst_name = release.get_archive_name()?;
    let dst_path = app.download_cache.get_path(&dst_name);

    if dst_path.exists() {
        println!("Already downloaded");
    } else {
        download(&release.url, &dst_path)?;
    }
    // TODO verify checksum
    unpack(&dst_path, &release.binaries, &app.bin_dir)?;

    Ok(())
}
