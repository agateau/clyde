# Creating a package

## Introduction

This document explains how to create a package for an imaginary application called namcap.

## Setup

We start by cloning the [Clyde store repository][store-repo].

[store-repo]: https://github.com/agateau/clyde-store

Then we create a branch for your package.

Then we create a file named namcap.yaml in the repository, with the following content:

```yaml
name: namcap
description: A one line description of the namcap app
homepage: https://namcap.example.com
```

## Adding release assets

Now we need to add release assets. Clyde provides a tool to help with this: `clydetools add-assets`.

Let's say namcap provides the following archives for version 1.2.3:

- <https://namcap.example.com/download/namcap-1.2.3-x86_64-linux.tar.gz>
- <https://namcap.example.com/download/namcap-1.2.3-x86_64-windows.zip>
- <https://namcap.example.com/download/namcap-1.2.3-x86_64-darwin.tar.gz>
- <https://namcap.example.com/download/namcap-1.2.3-aarch64-darwin.tar.gz>

To use all these archives for the 1.2.3 release, run `clydetools add-assets` like this:

```
clydetools add-assets namcap.yaml 1.2.3 \
    https://namcap.example.com/download/namcap-1.2.3-x86_64-linux.tar.gz \
    https://namcap.example.com/download/namcap-1.2.3-x86_64-windows.zip \
    https://namcap.example.com/download/namcap-1.2.3-x86_64-darwin.tar.gz \
    https://namcap.example.com/download/namcap-1.2.3-aarch64-darwin.tar.gz
```

`clydetools add-assets` downloads all archives, computes their sha256 checksum, and adds them as builds for the 1.2.3 release to `namcap.yaml`.

Our `namcap.yaml` should now look like this:

```yaml
name: namcap
description: A one line description of the namcap app
homepage: https://namcap.example.com
releases:
  1.2.3:
    aarch64-macos:
      url: https://namcap.example.com/download/namcap-1.2.3-aarch64-darwin.tar.gz
      sha256: some_sha256_checksum
    x86_64-linux:
      url: https://namcap.example.com/download/namcap-1.2.3-x86_64-linux.tar.gz
      sha256: some_sha256_checksum
    x86_64-macos:
      url: https://namcap.example.com/download/namcap-1.2.3-x86_64-macos.tar.gz
      sha256: some_sha256_checksum
    x86_64-windows
      url: https://namcap.example.com/download/namcap-1.2.3-x86_64-windows.zip
      sha256: some_sha256_checksum
```

## Adding installs

It's time to add an `installs` entry so that Clyde knows which files should go where.

Let's assume the archive content is the following:

```yaml
namcap-1.2.3-$arch-$os/
  namcap (namcap.exe on Windows)
  README.md
  LICENSE
  doc/
    GUIDE.md
    namcap.1
```

This is what the installation must do:

- `namcap` (`namcap.exe` on Windows) must go to the `bin` directory
- `namcap.1` must go to the `share/man/man1` directory
- The other files must go to the package documentation directory (`share/doc/namcap`)

We can do this with an `installs` entry like this:

```yaml
installs:
  1.2.3:
    any-any:
      strip: 1
      files:
        namcap${exe_ext}: bin/
        doc/namcap.1: share/man/man1/
        README.md: ${doc_dir}
        LICENSE: ${doc_dir}
        doc/GUIDE.md: ${doc_dir}
```

Let's unpack this. We start with the version number: it says Clyde that version 1.2.3 or later of the namcap package must follow these instructions (This means if we later add a new version of the application but whose archive content is organized the same way, we don't need to add another entry to the `installs` entry).

Then there is an `any-any` entry: the first `any` is for the architecture (x86_64, aarch64...). Since we do not have architecture-specific rules, we use `any`. The second `any` is for the OS (linux, macos, windows). Again since we do not have OS-specific rules, we use `any`.

Then `strip: 1` tells Clyde to skip the first subdirectory of the archive (the `namcap-1.2.3-$arch-$os` part).

Finally, the `files` part defines where each file go. There are a few subtleties here:

- For the executable we use the `${exe_ext}` variable to transparently handle the fact that Windows binaries have a ".exe" extension.
- To install to a directory, it *must* end with a `/`, writing `doc/namcap.1: share/man/man1` would mean moving `namcap.1` to `share/man` and renaming it to `man1`.
- To install to the documentation directory, we can use the `${doc_dir}` variable.

For more details on the file format, have a look at the [package-file-format.md](package-file-format.md) document.

## Testing it

We are done, time to test the package. Run `clyde install namcap.yaml` and see if it works as expected.

If you don't want to "pollute" your installation, you can setup a separate Clyde home by defining the `CLYDE_HOME` environment variable:

```
export CLYDE_HOME=/tmp/my-test-home
clyde setup
. $CLYDE_HOME/scripts/activate.sh
clyde install namcap.yaml
```

If some files are not at the right place, uninstall the package with `clyde uninstall namcap` (*not* `namcap.yaml`!) and reinstall.

## Sharing it

Congratulations! Our package is ready, you can now commit your new file and open a pull request to add your package to the store.
