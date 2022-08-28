// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;
use crate::table::Table;

pub fn list(app: &App) -> Result<()> {
    let table = Table::new(&[40, 12, 12]);
    table.add_row(&["Package", "Installed", "Requested"]);
    table.add_separator();
    for info in app.database.get_installed_packages()? {
        table.add_row(&[
            &info.name,
            &info.installed_version.to_string(),
            &info.requested_version.to_string(),
        ]);
    }
    Ok(())
}
