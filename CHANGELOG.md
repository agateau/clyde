# Changelog

## 0.6.0 - 2023-12-28

### Added

- Clyde now ships with auto-completion for Bash, Elvish, Fish, PowerShell and Zsh.

- Clyde packages can now install Fish completion using the new `${fish_comp_dir}` variable.

- When downloading from GitHub, `clydetools` now uses a GitHub token if one is set in `$CLYDE_GITHUB_TOKEN` or in `$GITHUB_TOKEN`.

- `clydetools` now has a short documentation (docs/clydetools.md).

- `clyde list` and `clyde show` can now output their results in JSON.

- The package format now supports a new type of fetcher: `ScriptFetcher`. This fetcher uses the [Boa JavaScript engine](https://boajs.dev/) to allow fetching from many different sources.

### Changed

- `clyde setup` now installs Clyde itself inside the newly created Clyde home, making initial setup simpler (#149).

- When a new version of Clyde is available, `clyde upgrade` won't install any other package than the `clyde` package, ensuring install or upgrade of packages are done with the latest version of Clyde.

- Download messages have been simplified: they no longer show the number of attempts on the first attempt.

- `clydetools fetch` now recognizes `64-Bit` and `32-Bit` as x86_64 and x86 architectures.

- `clyde upgrade` now list packages which cannot be upgraded because they are pinned.

- `clydetools fetch` does not fail anymore to extract version numbers from tags with app name prefixes.

### Deprecated

- The `$CLYDE_INST_DIR` environment variable is now deprecated. `$CLYDE_HOME` is now defined by the activation script, so one can use `$CLYDE_HOME/inst` instead.

### Fixed

- `clyde search` is now more robust: it won't fail if a package file cannot be parsed, and won't try to load the `.pre-commit-config.yaml` file as a package.

## 0.5.0 - 2023-03-20

### Added

- `clydetools fetch` can now fetch updates for packages hosted on gitlab.com.

- `clyde setup` learned the `--url` option to define the URL of the store.

- Clyde packages can now install Bash and Zsh completion using the new `${bash_comp_dir}` and `${zsh_comp_dir}` variables (#20).

- `clyde install` learned the `--reinstall` option.

### Changed

- `clydetools` learned new patterns to extract arch-os from URLs.

- `clyde setup` now creates a shallow clone of the store, making it faster (#17).

- When a download times out, Clyde now retries 2 times before giving up.

### Fixed

- `clydetools fetch` could sometimes confuse cached assets if their name did not include the version number.

### Package format change

- A new entry has been added: `fetcher`. It makes it possible for the package to define the fetcher to use, and default values for the architecture and/or OS.

## 0.4.1 - 2022-11-30

### Fixed

- Test commands can now use the `${exe_ext}` variable, making them more useful to test Windows packages.

- `clydetools fetch` no longer adds a release if no assets can be found.

- `clydetools` no longer adds empty `tests` and `extra-files` entries.

## 0.4.0 - 2022-11-19

### Added

- It is now possible for assets to be cross-os, cross-arch or both (#29).

### Changed

- `clydetools check` now detects if a package release contains no assets.

### Fixed

- Clyde now ensures only one command is running at a time on a given installation (#3).

### Package format change

- It is now possible to ship extra files with a package. This is useful to provide icon or .desktop files (#9).

- Installs can now define test commands using the new `installs.<version>.<arch-os>.tests` entry (#7).

## 0.3.1 - 2022-09-07

### Changed

- `clydetools add-assets` and `clydetools fetch` no longer download more than one URL per arch-os.

### Fixed

- `clyde show -l <package>` now works even for packages installed from outside the store.

- When `clydetools` writes package files, it no longer writes `strip: 0`.

- Unpacking of single-file executables compressed with xz now works as expected.

## 0.3.0 - 2022-08-28

### Added

- `clydetools add-assets` learned to recognize more architectures.

- Clyde can now unpack single-file archives (#69).

### Changed

- `clyde install` and `clyde uninstall` can now install/uninstall multiple packages at once (#2).

- `clyde install` now deletes downloaded assets after install has finished.

- Clyde output is nicer: command details are indented and `clyde list` prints packages as a table (#12).

- `clydetools add-build` has been renamed to `clydetools add-assets`.

### Package format change

- It's now possible to refer to the asset name in the `files` entries, using the new `${asset_name}` variable (#65).

- The `strip` field is now optional, and defaults to 0 (#10).

- A new field has been added: `repository` (#67).

## 0.2.1 - 2022-07-28

### Fixed

- Fixed bug which caused `./clyde install clyde` to fail (#58).

## 0.2.0 - 2022-07-26

### Added

- Clyde can now resume interrupted downloads.

### Changed

- Clyde implements downloading itself, it no longer requires curl (#5).

- Clyde now shows a more helpful message if git or tar are not available.

### Fixed

- Clyde now knows how to uninstall or upgrade itself on Windows.

- `clydetools add-build` now skips unsupported file formats like .deb, .rpm or .msi.

- When uninstalling packages containing symbolic links, such as node16, Clyde would sometimes fail to delete the package symbolic links.

## 0.1.0 - 2022-07-15

### Added

- Initial pre-release.
