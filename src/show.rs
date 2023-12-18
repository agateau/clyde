// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use serde_json::json;

use crate::app::App;

fn get_file_list(app: &App, package_name: &str) -> Result<Vec<String>> {
    let fileset = app.database.get_package_files(package_name)?;
    let mut files = Vec::from_iter(fileset);
    files.sort();
    Ok(files
        .iter()
        .map(|x| app.install_dir.join(x).display().to_string())
        .collect())
}

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
    let files = get_file_list(app, package_name)?;
    for file in files {
        println!("{}", file);
    }
    Ok(())
}

fn show_as_json(app: &App, package_name: &str, list: bool) -> Result<()> {
    let db = &app.database;
    let package = app.store.get_package(package_name)?;
    let installed_version = db
        .get_package_version(&package.name)?
        .map(|x| x.to_string());

    let available_versions: Vec<_> = package
        .releases
        .iter()
        .map(|(version, builds)| {
            let mut arch_os_list = Vec::from_iter(builds.keys().map(|x| format!("{x}")));
            arch_os_list.sort();
            json!({
                "version": version.to_string(),
                "arch_os": arch_os_list,
            })
        })
        .collect();

    let mut value = json!({
        "name": package.name,
        "description": package.description,
        "homepage": package.homepage,
        "repository": package.repository,
        "installed_version": installed_version,
        "available_versions": available_versions,
    });
    if list {
        value["files"] = get_file_list(app, &package.name)?.into()
    }
    println!("{}", value);
    Ok(())
}

pub fn show_cmd(app: &App, app_name: &str, json: bool, list: bool) -> Result<()> {
    if json {
        show_as_json(app, app_name, list)
    } else if list {
        show_files(app, app_name)
    } else {
        show_details(app, app_name)
    }
}
