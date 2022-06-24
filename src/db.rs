use std::collections::HashSet;
use std::include_str;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, Result as RusqliteResult};
use semver::{Version, VersionReq};

pub struct Database {
    conn: Connection,
}

pub struct PackageInfo {
    pub name: String,
    pub installed_version: Version,
    pub requested_version: VersionReq,
}

impl Database {
    pub fn new_from_path(db_path: &Path) -> Result<Database> {
        let conn = Connection::open(&db_path)?;

        Ok(Database { conn })
    }

    pub fn new_in_memory() -> Result<Database> {
        let conn = Connection::open_in_memory()?;

        Ok(Database { conn })
    }

    pub fn create(&self) -> Result<()> {
        self.conn.execute_batch(include_str!("create_db.sql"))?;
        Ok(())
    }

    pub fn get_package_version(&self, package: &str) -> Result<Option<Version>> {
        let row: RusqliteResult<String> = self.conn.query_row(
            "SELECT installed_version FROM installed_package
            WHERE name = ?",
            [&package],
            |row| row.get(0),
        );

        match row {
            Ok(version) => Ok(Some(Version::parse(&version)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(x) => Err(anyhow!(
                "Failed to get package version for {}: {}",
                package,
                x
            )),
        }
    }

    pub fn add_package(
        &self,
        package: &str,
        installed_version: &Version,
        requested_version: &VersionReq,
        files: &HashSet<PathBuf>,
    ) -> Result<()> {
        let installed_version_str = installed_version.to_string();
        let requested_version_str = requested_version.to_string();

        let tx = self.conn.unchecked_transaction()?;
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

    pub fn remove_package(&self, package: &str) -> Result<()> {
        self.conn
            .execute("DELETE from installed_package WHERE name = ?", [&package])?;
        Ok(())
    }

    pub fn get_package_files(&self, package: &str) -> Result<HashSet<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM installed_file WHERE package_name = ?")?;
        let mut rows = stmt.query([&package])?;

        let mut files = HashSet::<PathBuf>::new();
        while let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let path = PathBuf::from(name);
            files.insert(path);
        }
        Ok(files)
    }

    pub fn get_installed_packages(&self) -> Result<Vec<PackageInfo>> {
        let mut packages: Vec<PackageInfo> = Vec::<PackageInfo>::new();
        let mut stmt = self
            .conn
            .prepare("SELECT name, installed_version, requested_version FROM installed_package order by name")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let installed_version: String = row.get(1)?;
            let requested_version: String = row.get(2)?;
            packages.push(PackageInfo {
                name,
                installed_version: Version::parse(&installed_version)?,
                requested_version: VersionReq::parse(&requested_version)?,
            });
        }
        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_package_adds_version_files() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        let package = "pkg";
        let installed_version = Version::parse("1.2.3").unwrap();
        let requested_version = VersionReq::parse("1.2.*").unwrap();
        let files = HashSet::<PathBuf>::from([PathBuf::from("bin/p"), PathBuf::from("share/p")]);

        // WHEN add_package() is called
        let result = db.add_package(&package, &installed_version, &requested_version, &files);

        // THEN it succeeds
        assert!(result.is_ok(), "{:?}", result);

        // AND the package is there
        assert!(db.get_package_version(&package).unwrap() == Some(installed_version));

        // AND the files are there
        assert!(db.get_package_files(&package).unwrap() == files);
    }

    #[test]
    fn get_package_version_returns_none_if_package_is_not_installed() {
        // GIVEN an empty database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // WHEN get_package_version() is called
        let result = db.get_package_version("not_there");

        // THEN it returns none
        assert!(result.unwrap() == None);
    }

    #[test]
    fn get_installed_packages_should_return_packages_in_correct_order() {
        // GIVEN a database
        let db = Database::new_in_memory().unwrap();
        db.create().unwrap();

        // AND 4 packages
        let installed_version = Version::parse("1.2.3").unwrap();
        let files = HashSet::<PathBuf>::new();
        for name in &["bob", "alice", "deborah", "carl"] {
            db.add_package(&name, &installed_version, &VersionReq::STAR, &files)
                .unwrap();
        }

        // WHEN get_installed_packages() is called
        // THEN it succeeds
        let packages = db.get_installed_packages().unwrap();

        // AND it returns the packages in alphabetical order
        let names: Vec<String> = packages.iter().map(|x| x.name.clone()).collect();
        assert_eq!(names, &["alice", "bob", "carl", "deborah"]);
    }
}
