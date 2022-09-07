# Changelog

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
