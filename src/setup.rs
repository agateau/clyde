// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::include_str;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};
use shell_words::quote;

use crate::app::App;
use crate::ui::Ui;

const SH_INIT: &str = include_str!("activate.sh.tmpl");

fn posix_shell_path_from_path(path: &Path) -> String {
    quote(path.to_str().unwrap()).to_string()
}

fn shell_path_from_path(path: &Path) -> Result<String> {
    if cfg!(not(target_os = "windows")) {
        return Ok(posix_shell_path_from_path(path));
    }

    let output = match Command::new("cygpath").arg(path).output() {
        Ok(x) => x,
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                // TODO Try to use anyhow context() here?
                return Err(anyhow!("Failed to run cygpath"));
            }
            // No cygpath executable, ignore the error, assume we are in an environment which does
            // not need it
            return Ok(posix_shell_path_from_path(path));
        }
    };
    if !output.status.success() {
        let code = output.status.code().unwrap();
        return Err(anyhow!(
            "cygpath exited with code {code}:\n{}",
            output.status.code().unwrap()
        ));
    }
    let cygpath_output = String::from_utf8(output.stdout).unwrap();
    Ok(quote(cygpath_output.trim()).to_string())
}

fn create_activate_script(app: &App) -> Result<String> {
    let install_dir = shell_path_from_path(&app.install_dir)?;
    let content = SH_INIT.replace("@INSTALL_DIR@", &install_dir);

    let scripts_dir = app.home.join("scripts");
    let script_path = scripts_dir.join("activate.sh");

    fs::create_dir_all(&scripts_dir)?;
    fs::write(&script_path, content)?;

    let shell_script_path = shell_path_from_path(&script_path)?;
    Ok(shell_script_path)
}

fn update_activate_script(ui: &Ui, home: &Path) -> Result<()> {
    let app = App::new(home)?;
    ui.info("Updating activation script");
    create_activate_script(&app)?;
    Ok(())
}

pub fn setup(ui: &Ui, home: &Path, update_scripts: bool) -> Result<()> {
    if update_scripts {
        return update_activate_script(ui, home);
    }

    if home.exists() {
        return Err(anyhow!("Clyde directory ({:?}) already exists, not doing anything. Delete it if you want to start over.",
            home));
    }
    ui.info(&format!("Setting up Clyde in {:?}", home));

    fs::create_dir_all(home)?;

    let app = App::new(home)?;

    app.store.setup()?;

    ui.info("Creating Clyde database");
    app.database.create()?;

    ui.info("Creating activation script");
    let shell_script_path = create_activate_script(&app)?;

    eprintln!("\nAll set! To activate your Clyde installation, add this line to your shell startup script:\n\n\
              . {shell_script_path}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use which::which;

    #[test]
    fn shell_path_from_path_should_quote_correctly() {
        if which("sh").is_err() {
            // Skip test, can't be run in this environment
            println!("Skipped");
            return;
        }
        for input_path in ["/foo bar", "/it's quoted"] {
            // GIVEN a path with characters requiring quoting
            let shell_path = shell_path_from_path(Path::new(input_path)).unwrap();

            // WHEN the result path is passed to a shell to run "echo -n $result_path"
            let echo_output = Command::new("sh")
                .arg("-c")
                .arg(format!("echo {shell_path}"))
                .output()
                .unwrap();

            // THEN the output of the shell is input_path
            let echo_path = String::from_utf8(echo_output.stdout).unwrap();
            let echo_path = echo_path.trim();
            assert_eq!(input_path, echo_path);
        }
    }

    #[test]
    #[cfg(windows)]
    fn shell_path_from_path_should_use_cygpath() {
        if which("cygpath").is_err() {
            // Skip test, can't be run in this environment
            println!("Skipped");
            return;
        }
        for (input, expected) in [
            (r"C:\foo", "/c/foo"),
            (r"C:\foo with spaces", "'/c/foo with spaces'"),
        ] {
            let output = shell_path_from_path(Path::new(input)).unwrap();
            assert_eq!(output, expected.to_string());
        }
    }
}
