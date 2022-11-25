// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::common::{self, ClydeYamlWriter};

#[test]
fn clydetools_check_run_test_commands() {
    let test_exe_name = format!("a_program_not_in_path${{exe_ext}}");

    // GIVEN a package file with 2 test commands
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let mut yaml_writer = ClydeYamlWriter::new("0.1.0");
    yaml_writer.exe_name = test_exe_name.clone();
    yaml_writer.add_test(&format!("{test_exe_name} help list"));
    yaml_writer.add_test(&format!("{test_exe_name} help install"));
    let package_path = yaml_writer.write(&temp_dir).unwrap();

    // WHEN `clydetools check` is run against the package file
    let mut cmd =
        common::create_clydetools_command(&["check", &package_path.to_string_lossy()], &temp_dir);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let output_summary = format!(
        "status: {}

== stdout =================
{stdout}

== stderr =================
{stderr}
",
        output.status
    );

    // THEN it succeeds
    assert!(output.status.success(), "{}", output_summary);

    // AND the test commands have been executed
    // The output of `clyde help <cmd>` contains the string `clyde <cmd> [OPTIONS]`
    assert!(
        stdout.contains(&format!(" list [OPTIONS]")),
        "{}",
        output_summary
    );
    assert!(
        stdout.contains(&format!(" install [OPTIONS]")),
        "{}",
        output_summary
    );
}

#[test]
fn clydetools_check_a_failing_test_command_should_fail_the_package() {
    // GIVEN a package file with a failing test command
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let mut yaml_writer = ClydeYamlWriter::new("0.1.0");
    yaml_writer.add_test("clyde${exe_ext} not-a-command");
    let package_path = yaml_writer.write(&temp_dir).unwrap();

    // WHEN `clydetools check` is run against the package file
    let status = common::run_clydetools(&["check", &package_path.to_string_lossy()], &temp_dir);

    // THEN it fails
    assert!(!status.success());
}
