# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

from conftest import run_clyde


def test_install_without_setup_show_message(monkeypatch):
    # GIVEN a non-existing Clyde home
    monkeypatch.setenv("CLYDE_HOME", "/does/not/exist")

    # WHEN running a Clyde command
    result = run_clyde("update", check=False)

    # THEN the command fails
    assert result.returncode != 0

    # AND the error message suggests running `clyde setup`
    assert "clyde setup" in result.stderr
