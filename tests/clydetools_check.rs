// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;

use crate::common;

const PACKAGE_DEFINITION: &str = "
name: starship
description: The minimal, blazing-fast, and infinitely customizable prompt for any
  shell!
homepage: https://starship.rs
repository: https://github.com/starship/starship
releases:
  1.11.0:
    aarch64-linux:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-aarch64-unknown-linux-musl.tar.gz
      sha256: 88de0f4431c0efa6b784e6e749b2cc016676a631e7e0665056627e49a367c69a
    aarch64-macos:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-aarch64-apple-darwin.tar.gz
      sha256: ebbf89fdf7eceba06b312d0118974a5196cbd24e08b73e86da14674eb840af3c
    aarch64-windows:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-aarch64-pc-windows-msvc.zip
      sha256: 153a81e1aa3c736a6b6b7924470afbb2ade1ad8bcf9efb36193c8c21e0a4c2c2
    x86-linux:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-i686-unknown-linux-musl.tar.gz
      sha256: 08fc670d47af8ec2741aaec476bf529458827bd4b5cf2c31685edcf5dd020f06
    x86-windows:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-i686-pc-windows-msvc.zip
      sha256: 70f1492d669fd315f06d2d5941d80791c027141e69a364e3e1d20a4c180556a6
    x86_64-linux:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-x86_64-unknown-linux-musl.tar.gz
      sha256: 5bbe15596996a123f53f54f9687e9410dbec7ac0917e2ee033a3ddd6604a946e
    x86_64-macos:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-x86_64-apple-darwin.tar.gz
      sha256: 55138c8c40ea1fceb0360dc8d184e9d370183c121cd1fe2272507a3d12be51f6
    x86_64-windows:
      url: https://github.com/starship/starship/releases/download/v1.11.0/starship-x86_64-pc-windows-msvc.zip
      sha256: b8d58ca4119dc37f3647c3018e59d1b1bbbd06cfa782ebfa9e4be3e678af0c15
installs:
  1.9.1:
    any-any:
      files:
        starship${exe_ext}: bin/
      tests:
";

fn create_package_definition(tests: &[&str]) -> String {
    let mut definition = PACKAGE_DEFINITION.to_string();
    for test in tests {
        definition.push_str(&format!("        - {}\n", test));
    }
    definition
}

#[test]
fn clydetools_check_run_test_commands() {
    // GIVEN a package file with 2 test commands
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let package_path = temp_dir.join("starship.yaml");
    let test_output1 = temp_dir.join("test-output1");
    let test_output2 = temp_dir.join("test-output2");
    let package_definition = create_package_definition(&[
        &format!(
            "starship${{exe_ext}} --version > {}",
            test_output1.to_string_lossy()
        ),
        &format!(
            "starship${{exe_ext}} --help > {}",
            test_output2.to_string_lossy()
        ),
    ]);
    fs::write(&package_path, package_definition).unwrap();

    // WHEN `clydetools check` is run against the package file
    let status = common::run_clydetools(&["check", &package_path.to_string_lossy()]);

    // THEN it succeeds
    assert!(status.success());

    // THEN the test commands have been executed
    assert!(test_output1.exists());
    assert!(test_output2.exists());
}

#[test]
fn clydetools_check_a_failing_test_command_should_fail_the_package() {
    // GIVEN a package file with a failing test command
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let package_path = temp_dir.join("starship.yaml");
    let package_definition = create_package_definition(&[&"exit 1"]);
    fs::write(&package_path, package_definition).unwrap();

    // WHEN `clydetools check` is run against the package file
    let status = common::run_clydetools(&["check", &package_path.to_string_lossy()]);

    // THEN it fails
    assert!(!status.success());
}
