// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;

use clyde::app::App;

use crate::common::{self, ClydeYamlWriter};

#[test]
fn clyde_can_upgrade_itself() {
    // GIVEN a store with Clyde installed
    let clyde_home = assert_fs::TempDir::new().unwrap();

    let store_dir = clyde_home.join("store");
    fs::create_dir(&store_dir).unwrap();

    ClydeYamlWriter::new("0.1.0").write(&store_dir).unwrap();

    let app = App::new(&clyde_home).unwrap();
    app.database.create().unwrap();

    common::run_clyde(&clyde_home, &["install", "clyde"]);

    // WHEN a new version of Clyde is available
    ClydeYamlWriter::new("0.2.0").write(&store_dir).unwrap();

    // AND the user runs `clyde install clyde`, using the installed clyde executable, meaning
    // the executable has to replace itself
    // THEN clyde updates itself without problems
    common::run_clyde(&clyde_home, &["install", "clyde"]);
}
