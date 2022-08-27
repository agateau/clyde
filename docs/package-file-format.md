# Package file format

This document describes the file format used by Clyde.

Clyde packages are defined as YAML files.

## Meta information

```yaml
name: foobar
description: Foo Bar Baz
homepage: https://foobar.example.com
```

## Releases

Releases are stored in the `releases` mapping.

Each entry is a version, which itself contains entries for each arch-os asset.

Each asset as two entries: `url` and `sha256`.

```yaml
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
```

## Installs

Next is the "installs" entry. This entry tells Clyde how to install the downloaded asset.

`installs` is a mapping which contain entries for the supported versions. There is not necessarily one entry for each version: the `clyde install` command uses the entry with highest version which is lower than or equal to the version to install.

This means that if a package have `installs` entries for version 1.2.0 and 1.3.0, then installing 1.3.4 would use the 1.3.0 entries. Installing 1.2.4 would use the 1.2.0 entries.

Each version entry then contains arch-os entries. The arch and/or OS part can be set to `any` if install instructions are independent of the arch and/or the OS.

Each arch-os entry contain a `strip` entry and a `files` entry, as demonstrated below.

```yaml
installs:
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

Packages must follow these rules:
- install binaries in bin
- install man pages in share/man
- install documentation in `share/doc/<package_name>` (use `${doc_dir}` for this, see "Variables" section)

### Variables

The source and destination parts of the `files` mapping supports variables. A variable can be used with the `${variable_name}` syntax.

The following variables are available:

- `${asset_name}`: Name of the unpacked asset if the asset is a single-file asset. A single-file asset is an asset which is either the package executable, or a compressed version of it, compressed with gzip, bzip2 or xz. This variable is only available if the asset is a single-file asset.
- `${doc_dir}`: Directory storing the package documentation. Set to "share/doc/<package_name>/".
- `${exe_ext}`: Executable extension for the target OS. Set to ".exe" on Windows and "" on other OSes.
