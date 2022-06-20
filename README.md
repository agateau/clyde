# Clyde

Clyde is a package manager for prebuilt applications.

## Motivation

You want to install the latest version of tools like ripgrep, fd or gh, but:

- They are not available in your distribution, or the available versions are too old, and you don't want to mess up your system
- You don't want to have to think about where to install them, add them to $PATH, make their man page available, make auto-completion work…
- You don't want to remember how you installed them when it's time to update them

Maybe you don't have root access on the machine where you need these tools, so installing system packages is not an option.

Maybe you want to pin the version of one tool.

Maybe you are concerned about supply-chain attacks? (see Security section)

## Getting started

1. Download a Clyde binary (Clyde can update itself, but it needs to be installed manually first)

2. Run `clyde setup`

3. Add the created activation script to your shell startup script

4. Restart your shell

You are now ready to use Clyde. Let's install ripgrep:

```
clyde install ripgrep
```

Check it works:

```
rg --help
```

Check you can read its man page:

```
man rg
```

## Commands

### `clyde setup`

Setup Clyde: install Clyde store, and create an activation script. All changes are done in  "Clyde prefix" (see "Folder hierarchy" section)

### `clyde update`

Update Clyde store.

### `clyde install foobar[@version]`

Install `foobar` package, following the `@version` specifier if set.

### `clyde remove foobar`

Uninstalls the `foobar` package.

### `clyde show foobar`

Shows details about `foobar` package.

## Security

Is Clyde more secure than `curl <url> | bash`?

Yes, but it still requires you to be careful.

It is more secure in that the Clyde store contains the sha256 checksum of all archives, making it more complicated for an attacker to trick you into installing a corrupted archive.

This means if an attacker takes over the GitHub account of an app developer and replace some release artifacts with others, Clyde will refuse to install them. It does not protect however from the case where the attacker releases a new version of the application. To protect against this you need to pin the version numbers.

Clyde does not sandbox the applications.

Clyde installs binaries produced by app developers, it does not rebuild them (unlike projects like [Homebrew](https://brew.sh)).

## Internals

### Clyde YAML format

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

### Folder hierarchy

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

Clyde store DB is checked out in $prefix/store.

### Installed DB

Installed DB is stored in $prefix/clyde.sqlite.

#### Tables

```
installed_package:
- *name*
- installed_version
- requested_version

installed_file:
- *path*
- package_name
```
