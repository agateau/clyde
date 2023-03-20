# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

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
