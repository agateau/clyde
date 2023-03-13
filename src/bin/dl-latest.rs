// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use clyde::arch_os::ArchOs;
use clyde::file_cache::FileCache;
use clyde::package::Package;
use clyde::ui::Ui;

#[derive(Debug, Parser)]
#[clap(name = "clydetools", version)]
pub struct Cli {
    package_file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let ui = Ui::default();

    let package = Package::from_file(&cli.package_file)?;
    let version = package.get_latest_version().unwrap();
    let asset = match package.get_asset(version, &ArchOs::current()) {
        Some(x) => x,
        None => {
            print!("No release assets available for {}", ArchOs::current());
            return Ok(());
        }
    };

    let cache = FileCache::new(&PathBuf::from("."));
    let asset_path = cache.download(&ui, &package.name, version, &asset.url)?;
    println!("{}", asset_path.display());

    Ok(())
}
