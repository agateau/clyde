## pinky YAML format

```yaml
name: foobar
description: Foo Bar Baz
releases:
  - version: 1.2.3
    arch: amd64
    os: linux
    url: https://example.com/foobar/foobar-{{ version }}-{{ os }}-{{ arch }}.tar.gz
    sha256: 1234567890abcdef
    binaries:
      - foobar-{{ version }}/foobar: foobar
```

## Folder hierarchy

Binaries are added to ~/.local/lib/pinky.

Pinky store DB is checked out in ~/.local/share/pinky/store.

Install DB stored in ~/.local/share/pinky/installed.yaml.

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
