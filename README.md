# Clyde

Clyde is a package manager for prebuilt applications.

It works on Linux, macOS and Windows.

[![Clyde demo](https://asciinema.org/a/629496.svg)](https://asciinema.org/a/629496)

## Motivation

You want to install the latest version of tools like ripgrep, fd or fzf, but:

- They are not available in your distribution, or the available versions are too old, and you don't want to mess up your system.
- You don't want to have to think about where to install them, add them to $PATH or make their man pages available.
- You don't want to remember how you installed them when it's time to update them.

You don't have root access on the machine where you need these tools, so installing system packages is not an option.

You want to pin the tool versions to create a reproducible platform.

You are concerned about supply-chain attacks (see Security section).

## Getting started

### Installing Clyde

To get started, you need to download the Clyde binary yourself. Clyde can update itself, but it needs to be installed manually first. You can either:

- Download a pre-built archive from the [releases page](http://github.com/agateau/clyde/releases).

- Download a pre-built archive of the `main` branch from <https://builds.agateau.com/clyde>.

- Build it yourself. Clyde is written in Rust, so if you have the Rust tool-chain installed, then you can install it with `cargo install clyde`.

Next, make sure git is installed: Clyde uses git to download and update the Clyde store (this requirement might go away in the future).

### Setting up your Clyde home

Clyde installs all applications in "Clyde home directory": a directory created in the default cache directory of your home directory.

Assuming you downloaded a Clyde archive, unpacked it and changed to its directory.

Run `./clyde setup`. This creates Clyde home directory, and clones the [Clyde Store](https://github.com/agateau/clyde-store) in it.

```
$ ./clyde setup
[I] Setting up Clyde in "/home/demo/.cache/clyde"
Cloning into '/home/demo/.cache/clyde/store'...
remote: Enumerating objects: 1790, done.
remote: Counting objects: 100% (1790/1790), done.
remote: Compressing objects: 100% (653/653), done.
remote: Total 1790 (delta 1132), reused 1745 (delta 1113), pack-reused 0
Receiving objects: 100% (1790/1790), 499.73 KiB | 123.00 KiB/s, done.
Resolving deltas: 100% (1132/1132), done.
[I] Creating Clyde database
[I] Creating activation script

All set! To activate your Clyde installation, add this line to your shell startup script:

. /home/demo/.cache/clyde/scripts/activate.sh
```

Add the created activation script to your shell startup script and restart your shell.

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

Check auto-completion works:

```
$ rg --regex<tab>
--regexp            -- specify pattern
--regex-size-limit  -- specify upper size limit of compiled regex
```

## Commands

### `clyde setup`

Setup Clyde: setup the Clyde store, and creates an activation script. All changes are done in the "Clyde prefix" (see "Folder hierarchy" section).

The Clyde store contains the list of all packages Clyde can install.

### `clyde search foobar`

Searches Clyde store for a package matching "foobar" in its name or description.

### `clyde install foobar[@version]`

Installs `foobar` package, following the `@version` restriction if set.

The `@version` syntax follows [Cargo's interpretation of Semantic Versioning][cargo-semver].

This makes the syntax a bit surprising: `clyde install foobar@1.2.3` can install an 1.2.4 version or even an 1.3.0 version, because Cargo considers them to be compatible.

To really pin a version you must use `foobar@=1.2.3`. To install the latest 1.2 version, use `'foobar@1.2.*'` or `foobar@~1.2`.

This syntax may change in the future.

[cargo-semver]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html

### `clyde uninstall foobar`

Uninstalls the `foobar` package. Can also be called as `clyde remove foobar`.

### `clyde show foobar`

Shows details about `foobar` package.

### `clyde update`

Updates Clyde store so that Clyde is aware of the availability of new packages or new versions of existing packages.

### `clyde upgrade`

Upgrades all packages to the latest version. If a package has been installed with an `@version` restriction, enforces it.

### `clyde doc foobar`

Shows the list of documentation files provided by the `foobar` package. Let you pick one and read it with the appropriate application.

`clyde doc` looks for a pager to display text files. You can set one in `$CLYDE_PAGER` or in `$PAGER`. If none of these variables is set, it falls back to an hard-coded list of pager programs.

## FAQ

### Is Clyde more secure than `curl <url> | bash`?

Yes, but it still requires you to be careful.

It is more secure in that Clyde checks the integrity of all downloaded archives (The Clyde store contains the sha256 checksum of all known archives), making it more complicated for an attacker to trick you into installing a corrupted archive.

This means if an attacker takes over the GitHub account of an app developer and replaces some release artifacts with others, Clyde will refuse to install them. It does not protect however from the case where the attacker releases a new version of the application. To protect against this you need to pin the version numbers.

Clyde does not sandbox the applications.

### Are Clyde packages built by Clyde?

No, Clyde installs binaries produced by app developers, it does not rebuild them (unlike projects like [Homebrew](https://brew.sh)).

This means that there is no guarantee that a package will run on your machine, even if Clyde installs it properly. This is especially true on old Linux installations: it is up to the app developer to provide binaries working on your system.

If a package used to work but the newer version does not, then you can pin the install to the latest working version, using the `@version` syntax.

### Where is the list of Clyde packages stored?

Clyde packages are defined in the [Clyde Store repository][store-repo]. `clyde setup` clones this repository on your machine. `clyde update` pulls the latest changes from it.

### How do I request a new package?

File an issue on the [Clyde store repository][store-repo].

### How do I add a new package?

Follow the [creating a package documentation](docs/creating-a-package.md).

[store-repo]: https://github.com/agateau/clyde-store

## Similar projects

There are other projects similar to Clyde. This section lists some of them, and the ways they differ from Clyde:

- [Homebrew](https://brew.sh/):
    - Binaries are built by Homebrew, not by application developers (not necessarily a bad thing, just a different approach).
    - No Windows support.
    - Unreliable support for pinned versions.

- [Hermit](https://cashapp.github.io/hermit/):
    - More geared maintaining a set of tools to build a project.
    - No package integrity checks.
    - No Windows support.
    - No man page integration.

- [Huber](https://github.com/innobead/huber):
    - No package integrity checks.
    - Only for GitHub projects.
    - No man page integration.
    - Default package list is hard-coded in the application.
