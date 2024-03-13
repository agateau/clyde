# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

import json

from pathlib import Path

from conftest import get_bin_path, run_clyde, run_in_clyde_home


def test_install_pinned_version(clyde_home):
    # GIVEN a Clyde home
    # WHEN running `clyde install starship@=1.10.2`
    run_clyde("install", "starship@=1.10.2")

    # THEN the starship binary exists
    bin_path = get_bin_path("starship")
    assert bin_path.exists()

    # AND `starship --version` prints the expected version number
    result = run_in_clyde_home("starship --version")
    assert "starship 1.10.2" in result.stdout


def test_uninstall_package(clyde_home):
    # GIVEN the starship package is installed
    starship_path = get_bin_path("starship")

    run_clyde("install", "starship")

    assert starship_path.exists()

    # WHEN running `clyde uninstall starship`
    run_clyde("uninstall", "starship")

    # THEN the starship binary no longer exists
    assert not starship_path.exists()


def test_reinstall_package(clyde_home):
    # GIVEN the starship package is installed
    starship_path = get_bin_path("starship")

    run_clyde("install", "starship")
    assert starship_path.exists()

    # AND its binary has been removed
    starship_path.unlink()
    assert not starship_path.exists()

    # WHEN running `clyde install --reinstall starship`
    run_clyde("install", "--reinstall", "starship")

    # THEN the binary is back
    assert starship_path.exists()


def test_install_cleans_after_itself_in_case_of_failure(clyde_home):
    # Get the list of glab files
    #
    # This test needs to use a package which:
    # - provide binaries for all supported arch-os
    # - contains more than one file
    run_clyde("install", "glab")
    xh_info = json.loads(run_clyde("show", "-l", "--json", "glab").stdout)
    paths = [Path(x) for x in xh_info["files"]]
    run_clyde("uninstall", "glab")

    for idx, existing_path in enumerate(paths):
        # GIVEN the glab package is not installed but one of its files exists
        existing_path.write_text("foo")
        other_paths = paths[:idx] + paths[idx + 1:]

        # WHEN one tries to install glab
        proc = run_clyde("install", "glab", check=False)

        # THEN it fails
        assert proc.returncode != 0

        # AND no files from the package have been installed
        for other_path in other_paths:
            assert not other_path.exists(), f"{existing_path=} {other_paths=}"

        # AND the existing file has been left untouched
        assert existing_path.read_text() == "foo"
        existing_path.unlink()
