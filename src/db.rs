use std::collections::HashSet;
use std::include_str;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{params, Connection};
use semver::{Version, VersionReq};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new_from_path(db_path: &Path) -> Result<Database> {
        let conn = Connection::open(&db_path)?;

        Ok(Database { conn })
    }

    pub fn create(&self) -> Result<()> {
        self.conn.execute_batch(include_str!("create_db.sql"))?;
        Ok(())
    }

    pub fn add_package(
        &mut self,
        package: &str,
        installed_version: &Version,
        requested_version: &VersionReq,
        files: &HashSet<PathBuf>,
    ) -> Result<()> {
        let installed_version_str = installed_version.to_string();
        let requested_version_str = requested_version.to_string();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO installed_package(name, installed_version, requested_version)
                    VALUES(?, ?, ?)",
            params![&package, &installed_version_str, &requested_version_str],
        )?;

        {
            let mut stmt =
                tx.prepare("INSERT INTO installed_file (path, package_name) VALUES (?, ?)")?;
            for file in files {
                stmt.execute(params![&file.to_str(), &package])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
