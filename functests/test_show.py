# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

from conftest import run_clyde


def test_show_uninstalled(clyde_home):
    # GIVEN a Clyde home
    # WHEN running `clyde show starship`
    result = run_clyde("show", "starship")

    # THEN the output contains `starship` homepage
    assert "Homepage: https://starship.rs" in result.stdout

    # AND there are no installed version
    assert "Installed version:" not in result.stdout


def test_show_installed(clyde_home):
    # GIVEN a Clyde home
    # AND starship is installed
    run_clyde("install", "starship")

    # WHEN running `clyde show starship`
    result = run_clyde("show", "starship")

    # THEN there is an installed version
    assert "Installed version:" in result.stdout
