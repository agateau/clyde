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
    any:
      # instructions for all arch-os

      # Ignore first level of directory
      strip: 1
      files:
        # Copy bin/foo to bin/foo
        bin/foo: bin/foo

        # This is the same as bin/bar: bin/bar
        bin/bar:

        # If man is a directory, its content is copied recursively to
        # share/man
        man: share/man

        # If the destination ends with a '/', it is created as a directory.
        # In this example README.md is copied to share/doc/foobar/README.md.
        README.md: share/doc/foobar/
    any-macos:
      # macOS special instructions
```

## Clyde store

Clyde package files are stored in the Clyde store, a git repository hosted at <https://github.com/agateau/clyde-store>. The `clyde setup` commands checkouts this repository inside Clyde prefix (see section below).

## Folder hierarchy

The default prefix is a `clyde` directory created in the cache directory. The location of this cache depends on your OS:
- Linux: `$HOME/.cache/clyde` by default
- Windows: `{FOLDERID_LocalAppData}/clyde/cache`
- macOS: `$HOME/Library/Caches/clyde`

The prefix can be defined using the `$CLYDE_PREFIX` environment variable.

Packages are all installed in $prefix/inst.

Packages must follow these rules:
- install binaries in $prefix/inst/bin
- install man pages in $prefix/inst/share/man
- install bash completion files in $prefix/inst/share/completion/bash
- install zsh completion files in $prefix/inst/share/completion/zsh

Clyde store is checked out in $prefix/store.

## Installed DB

Clyde stores information about the installed packages in an SQLite database. The database path is $prefix/clyde.sqlite.

The tables are defined in the [create_db.sql file](../src/create_db.sql).
