// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod install {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    fn run_clyde(clyde_home: &Path, args: &[&str]) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_clyde"));
        cmd.env("CLYDE_HOME", clyde_home);
        cmd.args(args);
        let status = cmd.status().expect("Failed to run Clyde");
        assert!(status.success());
    }

    fn setup_clyde(clyde_home: &Path) {
        run_clyde(clyde_home, &["setup"]);
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
        run_clyde(&clyde_home, &["install", "starship"]);

        // THEN the download dir is still empty
        assert!(is_dir_empty(&download_dir));
    }
}
