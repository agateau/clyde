// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::common::{self, ClydeYamlWriter};

#[test]
fn clydetools_check_run_test_commands() {
    let test_exe_name = "a_program_not_in_path${exe_ext}".to_string();

    // GIVEN a package file with 2 test commands
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let test_output1 = temp_dir.join("test-output1");
    let test_output2 = temp_dir.join("test-output2");

    let mut yaml_writer = ClydeYamlWriter::new("0.1.0");
    yaml_writer.exe_name = test_exe_name.clone();
    yaml_writer.add_test(&format!("{test_exe_name} --version > test-output1"));
    yaml_writer.add_test(&format!("{test_exe_name} --help > test-output2"));
    let package_path = yaml_writer.write(&temp_dir).unwrap();

    // WHEN `clydetools check` is run against the package file
    let status = common::run_clydetools(&["check", &package_path.to_string_lossy()], &temp_dir);

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

    let mut yaml_writer = ClydeYamlWriter::new("0.1.0");
    yaml_writer.add_test("exit 1");
    let package_path = yaml_writer.write(&temp_dir).unwrap();

    // WHEN `clydetools check` is run against the package file
    let status = common::run_clydetools(&["check", &package_path.to_string_lossy()], &temp_dir);

    // THEN it fails
    assert!(!status.success());
}
