# Internals

This document describes the internals of a Clyde installation prefix.

## Package file format

Clyde packages are defined as YAML files.

Here is an example package file, showing the schema used by Clyde package files.

```yaml
name: foobar
description: Foo Bar Baz
homepage: https://foobar.example.com
releases:
  "1.2.3":
    x86_64-linux:
      url: https://example.com/foobar/foobar-1.2.3-x86_64-linux.tar.gz
      sha256: 1234567890abcdef

    x86_64-macos:
      url: https://example.com/foobar/foobar-1.2.3-x86_64-macos.tar.gz
      sha256: 1234567890abcdef

    x86_64-windows
      url: https://example.com/foobar/foobar-1.2.3-x86_64-windows.zip
      sha256: 1234567890abcdef

    aarch64-macos:
      url: https://example.com/foobar/foobar-1.2.3-aarch64-macos.tar.gz
      sha256: 1234567890abcdef

  "1.2.1":
    x86_64-linux:
      url: https://example.com/foobar/foobar-1.2.1-x86_64-linux.tar.gz
      sha256: 1234567890abcdef

installs:
  # `install` command is going to use the highest version <= version to install.
  #
  # This means if you have files entries for 1.2.0 and 1.3.0, then installing
  # 1.3.4 would use the 1.3.0 files entries. Installing 1.2.4 would use the
  # 1.2.0 entries.
  "1.2.0":
    any-any:
      # instructions for all arch-os

      # Ignore first level of directory. Defaults to 0 if not set.
      strip: 1

      files:
        # Copy "bin/foo" to "bin/foo":
        bin/foo: bin/foo

        # This can be simplified to:
        bin/foo: bin/
        # Note that for the destination to be interpreted as a directory, it
        # *must* end with a '/'.

        # Or even simpler:
        bin/foo:

        # But if the package has a Windows version, then the executable will be
        # called foo.exe, so for portability it's better to use the ${exe_ext}
        # variable. It expands to ".exe" on Windows and "" on other OSes.
        bin/foo${exe_ext}:

        # In this example, assuming "man" is a directory, its content is copied
        # recursively to "share/man"
        man: share/man

        # To install documentation, you can use the ${doc_dir} variable, which
        # expands to "share/doc/<package_name>/".
        # In this example "README.md" is copied to "share/doc/foobar/README.md".
        README.md: ${doc_dir}
    any-macos:
      # macOS special instructions
```

### Variables

The source and destination parts of the `files` mapping supports variables. A variable can be used with the `${variable_name}` syntax.

The following variables are available:

- `${asset_name}`: Name of the unpacked asset if the asset is a single-file asset. A single-file asset is an asset which is either the package executable, or a compressed version of it, compressed with gzip, bzip2 or xz. This variable is only available if the asset is a single-file asset.
- `${doc_dir}`: Directory storing the package documentation. Set to "share/doc/<package_name>/".
- `${exe_ext}`: Executable extension for the target OS. Set to ".exe" on Windows and "" on other OSes.

## Clyde store

Clyde package files are stored in the Clyde store, a git repository hosted at <https://github.com/agateau/clyde-store>. The `clyde setup` commands checkouts this repository inside Clyde home (see section below).

## Folder hierarchy

The default location for Clyde home is created in your user cache directory. The exact default location depends on your OS:
- Linux: `$HOME/.cache/clyde`
- Windows: `$LOCALAPPDATA/clyde/cache`
- macOS: `$HOME/Library/Caches/clyde`

The home location can be overridden using the `$CLYDE_HOME` environment variable.

Packages are all installed in $CLYDE_HOME/inst.

Packages must follow these rules:
- install binaries in $CLYDE_HOME/inst/bin
- install man pages in $CLYDE_HOME/inst/share/man

Clyde store is checked out in $CLYDE_HOME/store.

## Installed files database

Clyde stores information about the installed packages in an SQLite database. The database path is $CLYDE_HOME/clyde.sqlite.

The tables are defined in the [create_db.sql file](../src/create_db.sql).
