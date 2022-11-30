// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::Path;

use crate::common;

fn setup_clyde(clyde_home: &Path) {
    common::run_clyde(clyde_home, &["setup"]);
}

fn is_dir_empty(dir: &Path) -> bool {
    let mut iter = fs::read_dir(dir).unwrap();
    iter.next().is_none()
}

#[test]
fn install_removes_downloaded_archive() {
    // GIVEN a Clyde setup
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let clyde_home = temp_dir.join("clyde");
    setup_clyde(&clyde_home);

    // AND an empty download dir
    let download_dir = clyde_home.join("download");
    assert!(download_dir.is_dir());
    assert!(is_dir_empty(&download_dir));

    // WHEN a package is installed
    common::run_clyde(&clyde_home, &["install", "starship"]);

    // THEN the download dir is still empty
    assert!(is_dir_empty(&download_dir));
}
