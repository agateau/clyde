# Clyde

Clyde is a package manager for prebuilt applications.

It works on Linux, macOS and Windows.

[Clyde demo](https://user-images.githubusercontent.com/3575/181382294-df105634-537e-4dab-906a-650107a73500.webm)

## Motivation

You want to install the latest version of tools like ripgrep, fd or fzf, but:

- They are not available in your distribution, or the available versions are too old, and you don't want to mess up your system.
- You don't want to have to think about where to install them, add them to $PATH or make their man pages available.
- You don't want to remember how you installed them when it's time to update them.

You don't have root access on the machine where you need these tools, so installing system packages is not an option.

You want to pin the tool versions to create a reproducible platform.

You are concerned about supply-chain attacks (see Security section).

## Installation

### Installing Clyde

To get started, you need to download the Clyde binary yourself. Clyde can update itself, but it needs to be installed manually first. You can either:

- Get an archive from the [releases page](http://github.com/agateau/clyde/releases).

- Get a main build from <https://builds.agateau.com/clyde>.

- Build it yourself. Clyde is written in Rust, so if you have the Rust tool-chain installed, then you can clone its source code and install it with `clyde install --path .`.

Next, make sure these tools are installed:

- git: to download and update the Clyde store
- tar: to unpack tar archives

This requirement list might get smaller in the future if more features are implemented internally.

## Getting started

Assuming the `clyde` binary is in your PATH.

1. Run `clyde setup`.

2. Add the created activation script to your shell startup script.

3. Restart your shell.

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

4. To ensure you always run the latest version of Clyde, install it with itself!

```
clyde install clyde
```

## Commands

### `clyde setup`

Setup Clyde: setup the Clyde store, and creates an activation script. All changes are done in the "Clyde prefix" (see "Folder hierarchy" section).

The Clyde store contains the list of all packages Clyde can install.

### `clyde update`

Updates Clyde store so that Clyde is aware of the availability of new packages or new versions of existing packages.

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

### `clyde search foobar`

Searches Clyde store for a package matching "foobar" in its name or description.

### `clyde upgrade`

Upgrades all packages to the latest version. If a package has been installed with an `@version` restriction, enforces it.

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
