// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;

pub fn list(app: &App) -> Result<()> {
    for info in app.database.get_installed_packages()? {
        println!(
            "{}: {} ({})",
            &info.name, info.installed_version, info.requested_version
        );
    }
    Ok(())
}
