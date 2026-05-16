// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Result};
use serde_json::json;

use crate::{app::App, cli::ShowMode, table::Table};

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
    Ok(())
}

fn show_releases(app: &App, package_name: &str) -> Result<()> {
    let package = app.store.get_package(package_name)?;

    let rows: Vec<[String; 3]> = package
        .releases
        .iter()
        .rev()
        .map(|(version, release)| {
            let added_at = match release.added_at {
                Some(x) => format!("{}", x.format("%Y-%m-%d %H:%M")),
                None => "".into(),
            };

            let mut arch_os_list = Vec::from_iter(release.assets.keys().map(|x| format!("{x}")));
            arch_os_list.sort();
            let arch_os_str = arch_os_list.join(", ");

            [version.to_string(), added_at, arch_os_str]
        })
        .collect();

    if rows.is_empty() {
        return Err(anyhow!("No releases, this should not happen."));
    }

    let arch_os_width = rows.iter().map(|x| x[2].len()).max().unwrap();

    let table = Table::new(&[10, 16, arch_os_width]);

    table.add_row(&["Version", "Added at", "CPU architecture - OS"]);
    table.add_separator();

    for row in rows {
        table.add_row(&row);
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

fn show_as_json(app: &App, package_name: &str) -> Result<()> {
    let db = &app.database;
    let package = app.store.get_package(package_name)?;
    let installed_version = db
        .get_package_version(&package.name)?
        .map(|x| x.to_string());

    let available_versions: Vec<_> = package
        .releases
        .iter()
        .map(|(version, release)| {
            let mut arch_os_list = Vec::from_iter(release.assets.keys().map(|x| format!("{x}")));
            arch_os_list.sort();
            json!({
                "version": version.to_string(),
                "arch_os": arch_os_list,
                "added_at": release.added_at,
            })
        })
        .collect();

    let value = json!({
        "name": package.name,
        "description": package.description,
        "homepage": package.homepage,
        "repository": package.repository,
        "installed_version": installed_version,
        "releases": available_versions,
        "files": json!(get_file_list(app, &package.name)?),
    });
    println!("{}", value);
    Ok(())
}

pub fn show_cmd(app: &App, app_name: &str, mode: ShowMode) -> Result<()> {
    if mode.json {
        show_as_json(app, app_name)
    } else if mode.files {
        show_files(app, app_name)
    } else if mode.releases {
        show_releases(app, app_name)
    } else {
        show_details(app, app_name)
    }
}
