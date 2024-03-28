# Package file format

This document describes the file format used by Clyde.

Clyde packages are defined as YAML files.

They can either use the "directory layout" or the "plain file" layout.

In the directory layout, the package file is: `<package_name>/index.yaml`.

In the plain file layout, the package file is: `<package_name>.yaml`.

The directory layout is supported since 0.4.0.

## Meta information

```yaml
name: foobar
description: Foo Bar Baz

# The public homepage
homepage: https://foobar.example.com

# Where the code can be downloaded. Optional.
repository: https://src.example.com
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

`installs` is a mapping containing entries for the supported versions. There is not necessarily one entry for each version: the `clyde install` command uses the entry with the highest version which is lower than or equal to the version to install.

This means that if a package has `installs` entries for version 1.2.0 and 1.3.0, then installing 1.3.4 would use the 1.3.0 entries. Installing 1.2.4 would use the 1.2.0 entries.

Each version entry then contains arch-os entries. The arch and/or OS parts can be set to `any` if install instructions are independent of the arch and/or the OS.

Each arch-os entry can contain the following entries:

- `files`: a mapping of files contained in the asset to the place where they should be installed. See below for more examples.
- `strip` (optional): the number of directories to ignore inside the asset. For example if all files of foo-1.0.tar.gz are inside a `foo-1.0` directory, set `strip` to 1 to tell Clyde that all entries in `files` are *inside* this directory. Defaults to 0.
- `extra_files` (optional, since 0.4.0): the directory of a package using the directory format can contain an `extra_files` directory to provide files to install in addition to the asset files. This can be useful to provide launcher scripts, icons, or .desktop files. In this case this entry is a mapping of files from the `extra_files` directory to the place where they should be installed.
- `tests` (optional, since 0.4.0): a list of commands to run to verify the package is correct. Used by `clydetools check`.

Here is an example of an `installs` entry:

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

      extra_files:
        # Copy "extra_files/bar" from within the package directory to "bin/bar"
        bar: bin/
    any-macos:
      # macOS special instructions
```

Packages must follow these rules:

- install binaries in `bin`
- install man pages in `share/man`
- install documentation in `share/doc/<package_name>` (use `${doc_dir}` for this, see "Variables" section)

If it is not possible to install the package files this way, then install all package files in `opt/<package_name>/`, add a launcher script to `extra_files` and install it in `bin`. Don't forget to make your launcher script executable.

### Variables

The source and destination parts of the `files` mapping support variables. A variable can be used with the `${variable_name}` syntax.

The following variables are available:

- `${asset_name}`: Name of the unpacked asset if the asset is a single-file asset. A single-file asset is an asset which is either the package executable, or a compressed version of it, compressed with gzip, bzip2 or xz. This variable is only available if the asset is a single-file asset.
- `${doc_dir}`: Directory storing the package documentation. Set to "share/doc/<package_name>/".
- `${bash_comp_dir}`: Where to install Bash completion files.
- `${fish_comp_dir}`: Where to install Fish completion files.
- `${zsh_comp_dir}`: Where to install Zsh completion files.
- `${exe_ext}`: Executable extension for the target OS. Set to ".exe" on Windows and "" on other OSes.

## fetcher

The optional `fetcher` mapping tells `clydetools fetch` how to fetch package updates. It looks like this:

```yaml
fetcher: !<type>
  # type-specific entries
```

Where `<type>` must be one of `Auto` (default), `GitHub`, `GitLab`, `Forgejo`, `Script` or `Off`.

Fetcher entries depend on their type, some entries are supported by many fetchers:

- `arch`: optional, set a default architecture. Useful when it cannot be deduced from the asset name.
- `os`: optional, set a default OS. Useful when it cannot be deduced from the asset name.

### GitHub fetcher

This fetcher accepts the following entries:

- `arch`
- `os`

### GitLab fetcher

This fetcher accepts the following entries:

- `arch`
- `os`

### Forgejo fetcher

This fetcher can fetch from any Forgejo-powered code forge. It accepts the following entries:

- `base_url`: required, the base URL of the forge to connect to.
- `arch`
- `os`

### Script fetcher

This fetcher gets the latest available version of a package by running a JavaScript script. The package must uses a directory layout. The fetcher looks for a file called `fetch.js` in the package directory and executes it. The fetch script must look for the latest available version of the package and return an object of the form:

```
{
  "version": $VERSION,
  "urls": [
    $URL1,
    $URL2,
    …
  ]
}
```

If there is an error it must return `null`.

The script can use the `httpGet(url) -> Response` function to synchronously send HTTP GET requests. The `Response` object contains two attributes:

`status`: the HTTP status of the response,
`text`: the text of the response.

## Environment variables

Clyde activation script defines a `$CLYDE_HOME` environment variable pointing to Clyde home. This means Clyde `opt` directory for example, can be referred to as `$CLYDE_HOME/inst/opt`. Launcher scripts installed via `extra_files` can make use of the `$CLYDE_HOME` environment variable to refer to a file installed in `$CLYDE_HOME/inst/opt/<package_name>`.
