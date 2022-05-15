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

Pinky database is checked out in ~/.local/share/pinky/db.

Install state stored in ~/.local/share/pinky/pinky.yaml.

`pinky.yaml`:

```yaml
packages:
  - name: $name
    installed_version: $installed_version
    requested_version: $requested_version
```

`$installed_version` is a copy of `version` field for the installed version.
`$requested_version` is the version number specified by the user when they called `pinky install foobar==version`

## Commands

### `pinky install foobar[==$version]`

1. Look for `foobar` in DB.
2. If installed:
    if $installed_version matches $version
        exit
    else:
        uninstall `foobar`
3. Download archive to temporary directory
4. Unpack archive
5. Copy binaries to `$PINKY_BINARY_DIR`
6. Update install status
