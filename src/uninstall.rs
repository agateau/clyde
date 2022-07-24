// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::App;

use anyhow::{anyhow, Context, Result};

fn prepend_underscore(path: &Path) -> PathBuf {
    let mut dst_file_name = OsString::from("_");
    dst_file_name.push(
        path.file_name()
            .unwrap_or_else(|| panic!("{path:?} should have a file name")),
    );
    path.with_file_name(dst_file_name)
}

/// Like Path::exists(), but returns true if the argument is a broken symbolic link
fn path_exists(path: &Path) -> bool {
    path.is_symlink() || path.exists()
}

pub fn uninstall(app: &App, package_name: &str) -> Result<()> {
    let db = &app.database;

    let installed_version = match db.get_package_version(package_name)? {
        Some(x) => x,
        None => {
            return Err(anyhow!("Package {} is not installed", package_name));
        }
    };

    eprintln!("Removing {} {}...", &package_name, installed_version);

    let current_exe_path = env::current_exe().context("Can't find path to current executable")?;

    for file in db.get_package_files(package_name)? {
        let path = app.install_dir.join(file);
        if !path_exists(&path) {
            eprintln!("Warning: expected {:?} to exist, but it does not", &path);
            continue;
        }
        if cfg!(windows) && path == current_exe_path {
            // Uninstalling Clyde itself is tricky on Windows, because we want to remove clyde.exe,
            // but it's currently running and Windows does not allow removing a running executable.
            // It is however possible to rename a running executable, so we rename it to _clyde.exe
            // and leave it there.
            // In the future it would be a good idea to look into really removing it.
            let dst_path = prepend_underscore(&path);
            eprintln!("Moving {path:?} to {dst_path:?}");
            fs::rename(&path, &dst_path)
                .with_context(|| format!("Failed to move {path:?} to {dst_path:?}"))?;
        } else {
            fs::remove_file(&path).with_context(|| format!("Failed to remove {path:?}"))?;
        }
    }
    db.remove_package(package_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    use semver::{Version, VersionReq};

    use crate::test_file_utils::*;

    #[test]
    fn uninstall_should_only_remove_the_package_files() {
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

        // WHEN uninstall() is called on `p2`
        let result = uninstall(&app, "p2");
        assert!(result.is_ok(), "{:?}", result);

        // THEN only `share/man/f1` file remains
        let expected = pathbufset_from_strings(&["share/man/f1"]);
        assert_eq!(list_tree(&app.install_dir).unwrap(), expected);

        // AND the package is no longer listed in the DB
        let result = db.get_package_files("p2").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    #[cfg(unix)]
    /// A broken symbolic link can happen during uninstallation if the link target is deleted
    /// before the link is. Since we can't define the order in which files are removed, in
    /// this test we use a package containing a truly broken symbolic link.
    fn uninstall_should_uninstall_broken_symbolic_links() {
        // GIVEN a Clyde home with a package containing a broken symbolic link
        let dir = assert_fs::TempDir::new().unwrap();
        let app = App::new(&dir).unwrap();
        let db = &app.database;
        db.create().unwrap();

        let bin_dir = app.install_dir.join("bin");
        let symbolic_link = bin_dir.join("foo");
        fs::create_dir_all(&bin_dir).unwrap();
        symlink(bin_dir.join("foo-real"), &symbolic_link).unwrap();
        assert!(symbolic_link.is_symlink());

        db.add_package(
            "foo",
            &Version::new(1, 0, 0),
            &VersionReq::STAR,
            &pathbufset_from_strings(&["bin/foo"]),
        )
        .unwrap();

        // WHEN uninstall() is called
        let result = uninstall(&app, "foo");

        // THEN it succeeds
        assert!(result.is_ok(), "{:?}", result);

        // AND the symbolic link is removed
        assert!(!symbolic_link.is_symlink());
    }

    #[test]
    fn test_path_exists() {
        let dir = assert_fs::TempDir::new().unwrap();
        let existing_file = dir.join("existing");
        let non_existing_file = dir.join("non_existing");

        fs::write(&existing_file, "").unwrap();

        assert!(path_exists(&existing_file));
        assert!(!path_exists(&non_existing_file));

        #[cfg(unix)]
        {
            let valid_symlink = dir.join("real_symlink");
            let broken_symlink = dir.join("non_existing_symlink");

            symlink(&existing_file, &valid_symlink).unwrap();
            symlink(&non_existing_file, &broken_symlink).unwrap();

            assert!(path_exists(&valid_symlink));
            assert!(path_exists(&broken_symlink));
        }
    }
}
