# Internals

This document describes the internals of a Clyde home.

## Clyde home location

The default location for Clyde home depends on your OS:
- Linux: `$HOME/.cache/clyde`
- Windows: `$LOCALAPPDATA/clyde/cache`
- macOS: `$HOME/Library/Caches/clyde`

The home location can be overridden using the `$CLYDE_HOME` environment variable.

## Folder hierarchy

- `$CLYDE_HOME`
    - `inst`: where files are installed
        - `bin`
        - `share`
    - `download`: where clyde downloads assets
    - `store`: Clyde store (see below)
    - `scripts`: activation scripts
    - `tmp_dir`: used while installing
    - `clyde.sqlite`: installed packages database (see below)

## Clyde store

Clyde package files are stored in the Clyde store, a git repository hosted at <https://github.com/agateau/clyde-store>. The `clyde setup` commands checkouts this repository inside Clyde home (see section below).

Clyde packages are defined as YAML files. The file format is described in [package-file-format.md](package-file-format.md).

## Installed packages database

Clyde stores information about the installed packages in an SQLite database.

The tables are defined in the [create_db.sql file](../src/create_db.sql).
