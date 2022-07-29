// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use crate::app::App;
use crate::ui::Ui;

pub fn update(app: &App, ui: &Ui) -> Result<()> {
    ui.info("Updating Clyde store");
    app.store.update()
}
