// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;

pub fn update(app: &App) -> Result<()> {
    eprintln!("Updating Clyde store");
    app.store.update()
}
