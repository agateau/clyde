# Clydetools

`clydetools` provide a set of commands for package maintainers.

## `add-assets <PACKAGE_FILE> <VERSION> [URLS]`

Downloads the assets from the specified URLs, compute their checksum and add an entry to the `releases` mapping of the package.

When downloading from GitHub, `clydetools add-assets` can make use of a GitHub token to avoid being rate-limited. The token is first looked for in `$CLYDE_GITHUB_TOKEN` and, if not set, in `$GITHUB_TOKEN`.

## `check <PACKAGE_FILES>`

For each package file, `clydetools check` runs some sanity checks and runs all defined tests.

## `fetch <PACKAGE_FILES>`

Looks for new versions of packages for which a fetcher is defined (see [package-file-format.md](package-file-format.md)). If it finds a new version, `clydetools fetch` gathers the URLs for all the assets and adds them to the package like `clydetools add-assets` would.
