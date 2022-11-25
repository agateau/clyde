// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::Result;

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

pub struct ClydeYamlWriter {
    version: String,
}

impl ClydeYamlWriter {
    pub fn new(version: &str) -> ClydeYamlWriter {
        ClydeYamlWriter {
            version: version.to_string(),
        }
    }

    pub fn write(self, store_dir: &Path) -> Result<PathBuf> {
        let clyde_path = env!("CARGO_BIN_EXE_clyde");
        let url = format!("file://{clyde_path}").replace('\\', "/");
        let sha256 = checksum::compute_checksum(Path::new(&clyde_path))?;

        let content = CLYDE_YAML_TEMPLATE
            .replace("@version@", &self.version)
            .replace("@arch_os@", &ArchOs::current().to_str())
            .replace("@url@", &url)
            .replace("@sha256@", &sha256);

        let clyde_yaml_path = store_dir.join("clyde.yaml");
        fs::write(&clyde_yaml_path, content)?;
        Ok(clyde_yaml_path)
    }
}

pub fn run_clyde(clyde_home: &Path, args: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clyde"));
    cmd.env("CLYDE_HOME", clyde_home);
    cmd.args(args);
    let status = cmd.status().expect("Failed to run Clyde");
    assert!(status.success());
}

pub fn run_clydetools(args: &[&str], cwd: &Path) -> ExitStatus {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clydetools"));

    // Add dir containing clyde binary to $PATH so that clydetools can run it
    let clyde_path = Path::new(env!("CARGO_BIN_EXE_clyde"));
    let clyde_dir = clyde_path.parent().unwrap();
    let path_var = env!("PATH");
    cmd.env(
        "PATH",
        format!("{}:{}", clyde_dir.to_string_lossy(), path_var),
    );

    cmd.current_dir(cwd);
    cmd.args(args);
    cmd.status().expect("Failed to run clydetools")
}
