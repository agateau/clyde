use std::fs;

use crate::app::App;

use anyhow::{anyhow, Result};

pub fn remove(app: &App, package_name: &str) -> Result<()> {
    let db = &app.database;

    if db.get_package_version(package_name)?.is_none() {
        return Err(anyhow!("Package {} is not installed", package_name));
    }

    eprintln!("Removing {}...", &package_name);

    for file in db.get_package_files(package_name)? {
        let path = app.install_dir.join(file);
        if path.exists() {
            fs::remove_file(&path)?;
        } else {
            eprintln!("Warning: expected {:?} to exist, but it does not", &path);
        }
    }
    db.remove_package(package_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use semver::{Version, VersionReq};

    use crate::test_file_utils::*;

    #[test]
    fn remove_should_only_remove_the_package_files() {
        // GIVEN a prefix with a `share/man/f1` file
        let dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(&dir).unwrap();
        let db = &app.database;
        db.create().unwrap();
        create_tree(&app.install_dir, &["share/man/f1"]);

        // AND a package `p2` containing a `bin/b2` and a `share/man/f2` file
        let package_files = pathbufset_from_strings(&["bin/b2", "share/man/f2"]);
        create_tree_from_path_set(&app.install_dir, &package_files);
        db.add_package(
            "p2",
            &Version::new(1, 0, 0),
            &VersionReq::STAR,
            &package_files,
        )
        .unwrap();

        // WHEN remove() is called on `p2`
        let result = remove(&app, "p2");
        assert!(result.is_ok(), "{:?}", result);

        // THEN only `share/man/f1` file remains
        let expected = pathbufset_from_strings(&["share/man/f1"]);
        assert_eq!(list_tree(&app.install_dir).unwrap(), expected);

        // AND the package is no longer listed in the DB
        let result = db.get_package_files("p2").unwrap();
        assert!(result.is_empty());
    }
}
