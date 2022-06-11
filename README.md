## pinky YAML format

```yaml
name: foobar
description: Foo Bar Baz
releases:
  "1.2.3":
    x86_64-linux:
      url: https://example.com/foobar/foobar-1.2.3-x86_65-linux.tar.gz
      sha256: 1234567890abcdef

  "1.2.1":
    x86_64-linux:
      url: https://example.com/foobar/foobar-1.2.1-x86_65-linux.tar.gz
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
        bin/foo-*: bin/foo
        # This is the same as bin/bar: bin/bar
        bin/bar:
        man/*: share/man
        README.md share/doc/foobar/README.md
    any-macos:
      # macOS special instructions
```

## Folder hierarchy

Default prefix is ~/.cache/pinky.

Packages are all installed in $prefix/inst.

Packages must follow these rules:
- install binaries in $prefix/inst/bin
- install man pages in $prefix/inst/share/man
- install bash completion files in $prefix/inst/share/completion/bash
- install zsh completion files in $prefix/inst/share/completion/zsh

Pinky store DB is checked out in $prefix/store.

Install DB stored in $prefix/installed.yaml.

`installed.yaml`:

```yaml
packages:
  - name: $name
    installed_version: $installed_version
    requested_version: $requested_version
```

`$installed_version` is a copy of `version` field for the installed version.
`$requested_version` is the version number specified by the user when they called `pinky install foobar==version`.

## Commands

### `pinky install foobar[==$version]`

1. Look for `foobar arch==$arch os==$os [version==$version]` in store DB.
2. If not found: exit with error.
3. Look for `foobar [version==$version]` in installed DB.
    if $installed_version matches $version
        exit
    else:
        uninstall `foobar`
4. Download archive to temporary directory.
5. Check archive checksum.
6. Unpack archive.
7. Copy binaries to `$PINKY_BINARY_DIR`.
8. Update installed DB.

### `pinky remove foobar`

1. Look for `foobar` in installed DB.
2. If not installed: exit with error.
3. Delete all binaries listed in foobar@version.
4. Remove `foobar` from installed DB.

### `pinky show foobar`

Shows details about `foobar` package.
