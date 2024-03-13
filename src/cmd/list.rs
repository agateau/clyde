// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::vec::Vec;

use crate::app::App;
use crate::db::PackageInfo;
use crate::table::Table;

use anyhow::Result;
use serde_json::{json, Value};

pub fn list_cmd(app: &App, json: bool) -> Result<()> {
    let packages = app.database.get_installed_packages()?;
    if json {
        list_as_json(&packages);
    } else {
        list_as_text(&packages);
    }
    Ok(())
}

fn list_as_text(packages: &[PackageInfo]) {
    let table = Table::new(&[40, 12, 12]);
    table.add_row(&["Package", "Installed", "Requested"]);
    table.add_separator();
    for info in packages {
        table.add_row(&[
            &info.name,
            &info.installed_version.to_string(),
            &info.requested_version.to_string(),
        ]);
    }
}

fn list_as_json(packages: &[PackageInfo]) {
    let names: Vec<Value> = packages
        .iter()
        .map(|x| {
            json!({
                "name": x.name.clone(),
                "installed_version": x.installed_version.to_string(),
                "requested_version": x.requested_version.to_string(),
            })
        })
        .collect();
    let array = json!(names);
    println!("{}", array);
}
