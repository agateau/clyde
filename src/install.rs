use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::app::App;
use crate::package::{Package, Release};

fn download(url: &str, dst_path: &Path) -> Result<()> {
    println!("Downloading {} to {:?}", url, dst_path);
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

    download(&release.url, &dst_path)?;

    Ok(())
}
