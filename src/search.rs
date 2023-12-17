// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;
use crate::ui::Ui;

pub fn search_cmd(app: &App, ui: &Ui, query: &str) -> Result<()> {
    let (results, errors) = app.store.search(query)?;
    if results.is_empty() {
        eprintln!("No packages found matching '{query}'");
    } else {
        for result in results {
            println!("{}: {}", result.name, result.description);
        }
    }
    for error in errors {
        ui.warn(&format!("{:#}", error));
    }
    Ok(())
}
