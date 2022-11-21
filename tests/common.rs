// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::process::{Command, ExitStatus};

pub fn run_clyde(clyde_home: &Path, args: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clyde"));
    cmd.env("CLYDE_HOME", clyde_home);
    cmd.args(args);
    let status = cmd.status().expect("Failed to run Clyde");
    assert!(status.success());
}

pub fn run_clydetools(args: &[&str]) -> ExitStatus {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clydetools"));

    // Add dir containing clyde binary to $PATH so that clydetools can run it
    let clyde_path = Path::new(env!("CARGO_BIN_EXE_clyde"));
    let clyde_dir = clyde_path.parent().unwrap();
    let path_var = env!("PATH");
    cmd.env(
        "PATH",
        format!("{}:{}", clyde_dir.to_string_lossy(), path_var),
    );
    println!("path={}", path_var);

    cmd.args(args);
    cmd.status().expect("Failed to run clydetools")
}
