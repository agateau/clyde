// SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use dialoguer::Select;
use glob_match::glob_match;
use shell_words;

use crate::app::App;
use crate::pager::find_pager;

#[derive(Debug, Clone, Copy)]
enum DocApp {
    Pager,
    Man,
    System,
}

lazy_static! {
    static ref APP_PATTERNS: Vec<(DocApp, &'static str)> = vec![
        (DocApp::Pager, "*.md"),
        (DocApp::Pager, "*.txt"),
        (DocApp::Pager, "LICENSE"),
        (DocApp::Pager, "CHANGELOG"),
        (DocApp::Pager, "COPYING"),
        (DocApp::Pager, "README.*"),
        (DocApp::Man, "*.[1-9]"),
        (DocApp::Man, "*.[1-9].gz"),
    ];
}

fn is_doc_file(path: &Path) -> bool {
    path.starts_with("share/doc") || path.starts_with("share/man")
}

fn get_doc_file_list(app: &App, package_name: &str) -> Result<Vec<PathBuf>> {
    let fileset = app.database.get_package_files(package_name)?;
    let mut files: Vec<_> = fileset.iter().filter(|x| is_doc_file(x)).cloned().collect();
    files.sort();
    Ok(files)
}

fn select_doc_file(app: &App, package_name: &str) -> Result<Option<PathBuf>> {
    let files = get_doc_file_list(app, package_name)?;
    if files.is_empty() {
        return Err(anyhow!(
            "Package {package_name} does not provide documentation"
        ));
    }

    let items: Vec<_> = files.iter().map(|x| x.display()).collect();

    let idx = Select::new().items(&items).default(0).interact_opt()?;

    let idx = match idx {
        Some(x) => x,
        None => return Ok(None),
    };

    let doc_file = app.install_dir.join(&files[idx]);

    Ok(Some(doc_file))
}

fn find_doc_app(doc_file: &Path) -> DocApp {
    let name = doc_file
        .file_name()
        .expect("A doc file should have a file name")
        .to_string_lossy();
    for (doc_app, pattern) in APP_PATTERNS.iter() {
        if glob_match(pattern, &name) {
            return *doc_app;
        }
    }

    DocApp::System
}

fn open_doc_file(doc_file: &Path) -> Result<()> {
    let app = find_doc_app(doc_file);

    let command = match app {
        DocApp::Pager => match find_pager() {
            Some(x) => x,
            None => {
                return Err(anyhow!("Could not find a pager, install one with Clyde, for example using `clyde install bat`"));
            }
        },
        DocApp::Man => "man".to_string(),
        DocApp::System => {
            return Ok(open::that(doc_file)?);
        }
    };

    // Split command
    let words = shell_words::split(&command)?;
    let mut iter = words.iter();
    let binary = iter
        .next()
        .ok_or_else(|| anyhow!("Pager command is empty"))?;
    let args: Vec<String> = iter.map(|x| x.into()).collect();

    Command::new(binary)
        .args(args)
        .arg(doc_file.as_os_str())
        .status()
        .map_err(|x| anyhow!("Failed to start command: {}: {}", command, x))?;
    Ok(())
}

pub fn doc_cmd(app: &App, package_name: &str) -> Result<()> {
    let db = &app.database;

    // Returns if package does not exist
    app.store.get_package(package_name)?;

    if db.get_package_version(package_name)?.is_none() {
        return Err(anyhow!("{} is not installed", package_name));
    }
    let doc_file = match select_doc_file(app, package_name)? {
        Some(x) => x,
        None => return Ok(()),
    };

    open_doc_file(&doc_file)
}
