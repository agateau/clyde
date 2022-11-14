// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::boxed::Box;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use single_instance::SingleInstance;

use crate::db::Database;
use crate::file_cache::FileCache;
use crate::store::{GitStore, Store};
use crate::ui::Ui;

const CLYDE_STORE_URL: &str = "https://github.com/agateau/clyde-store";

pub struct App {
    pub download_cache: FileCache,
    pub home: PathBuf,
    pub install_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub store_dir: PathBuf,
    pub store: Box<dyn Store>,
    pub database: Database,
}

impl App {
    pub fn find_home(ui: &Ui) -> Result<PathBuf> {
        if let Some(home) = env::var_os("CLYDE_HOME") {
            ui.info(&format!("Using {home:?} as Clyde home directory"));
            return Ok(Path::new(&home).to_path_buf());
        }

        if let Some(prefix_path) = ProjectDirs::from("", "", "clyde") {
            return Ok(prefix_path.cache_dir().to_path_buf());
        }

        Err(anyhow!("Could not find Clyde home directory"))
    }

    /// Make sure that for a given home directory, only one instance of Clippy is running at a time
    pub fn create_single_instance(home: &Path) -> Result<SingleInstance> {
        let name = home
            .to_str()
            .unwrap_or_else(|| panic!("{home:?} has a weird name."));

        let instance = SingleInstance::new(name)
            .unwrap_or_else(|x| panic!("Failed to check if instance is unique: {x}"));

        if !instance.is_single() {
            return Err(anyhow!("Another instance of Clyde is already running."));
        }
        Ok(instance)
    }

    /// Creates the app. It takes a home which *must* exist. This ensures no command
    /// can run if `clyde setup` has not been called.
    pub fn new(home: &Path) -> Result<App> {
        if !home.exists() {
            return Err(anyhow!(
                "Clyde home {:?} does not exist. Call `clyde setup` to create it.",
                home
            ));
        }
        let store_dir = home.join("store");
        let store = GitStore::new(CLYDE_STORE_URL, &store_dir);

        let db_path = home.join("clyde.sqlite");
        let database = Database::new_from_path(&db_path)?;

        let download_dir = home.join("download");
        fs::create_dir_all(&download_dir)?;

        Ok(App {
            download_cache: FileCache::new(&download_dir),
            home: home.to_path_buf(),
            install_dir: home.join("inst"),
            tmp_dir: home.join("tmp"),
            store_dir,
            store: Box::new(store),
            database,
        })
    }
}
