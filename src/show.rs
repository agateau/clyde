// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;

fn show_details(app: &App, package_name: &str) -> Result<()> {
    let db = &app.database;
    let package = app.store.get_package(package_name)?;
    println!("Name: {}", package.name);
    println!("Description: {}", package.description);
    println!("Homepage: {}", package.homepage);
    println!("Repository: {}", package.repository);

    if let Some(installed_version) = db.get_package_version(&package.name)? {
        println!("Installed version: {installed_version}");
    }

    println!();
    println!("Available versions:");
    for (version, builds) in package.releases.iter().rev() {
        let mut arch_os_list = Vec::from_iter(builds.keys().map(|x| format!("{x}")));
        arch_os_list.sort();
        let arch_os_str = arch_os_list.join(", ");
        println!("- {version} ({arch_os_str})");
    }
    Ok(())
}

fn show_files(app: &App, package_name: &str) -> Result<()> {
    let fileset = app.database.get_package_files(package_name)?;
    let mut files = Vec::from_iter(fileset);
    files.sort();
    for file in files {
        let path = app.install_dir.join(file);
        println!("{}", path.display());
    }
    Ok(())
}

pub fn show(app: &App, app_name: &str, list: bool) -> Result<()> {
    if list {
        show_files(app, app_name)
    } else {
        show_details(app, app_name)
    }
}
