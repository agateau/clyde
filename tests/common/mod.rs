// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::process::Command;

pub fn run_clyde(clyde_home: &Path, args: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clyde"));
    cmd.env("CLYDE_HOME", clyde_home);
    cmd.args(args);
    let status = cmd.status().expect("Failed to run Clyde");
    assert!(status.success());
}
