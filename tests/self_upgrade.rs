// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use std::env;
use std::fs;
use std::path::Path;

use anyhow::Result;

use clyde::app::App;
use clyde::arch_os::ArchOs;
use clyde::checksum;

const CLYDE_YAML_TEMPLATE: &str = "
        name: clyde
        description:
        homepage:
        releases:
          @version@:
            @arch_os@:
              url: @url@
              sha256: @sha256@
        installs:
          @version@:
            any:
              strip: 0
              files:
                clyde${exe_ext}: bin/
        ";

fn create_clyde_yaml(store_dir: &Path, version: &str) -> Result<()> {
    let clyde_path = env!("CARGO_BIN_EXE_clyde");
    let url = format!("file://{clyde_path}");
    let sha256 = checksum::compute_checksum(Path::new(&clyde_path))?;

    let content = CLYDE_YAML_TEMPLATE
        .replace("@version@", version)
        .replace("@arch_os@", &ArchOs::current().to_str())
        .replace("@url@", &url)
        .replace("@sha256@", &sha256);
    fs::write(store_dir.join("clyde.yaml"), content)?;
    Ok(())
}

#[test]
fn clyde_can_upgrade_itself() {
    // GIVEN a store with Clyde installed
    let clyde_home = assert_fs::TempDir::new().unwrap();

    let store_dir = clyde_home.join("store");
    fs::create_dir(&store_dir).unwrap();
    create_clyde_yaml(&store_dir, "0.1.0").unwrap();

    let app = App::new(&clyde_home).unwrap();
    app.database.create().unwrap();

    common::run_clyde(&clyde_home, &["install", "clyde"]);

    // WHEN a new version of Clyde is available
    create_clyde_yaml(&store_dir, "0.2.0").unwrap();

    // AND the user runs `clyde install clyde`, using the installed clyde executable, meaning
    // the executable has to replace itself
    // THEN clyde updates itself without problems
    common::run_clyde(&clyde_home, &["install", "clyde"]);
}
