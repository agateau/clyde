# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

import subprocess
from pathlib import Path

from conftest import get_bin_path, run_clyde, run_in_clyde_home, IS_WINDOWS


def test_install_without_setup_show_message(clyde_home, monkeypatch):
    clyde_bin_path = get_bin_path("clyde")

    # GIVEN a non-existing Clyde home
    monkeypatch.setenv("CLYDE_HOME", "/does/not/exist")

    # WHEN running a Clyde command
    # Note: we do not use run_clyde() here because it only works when $CLYDE_HOME points
    # to a valid Clyde home
    result = subprocess.run([clyde_bin_path, "update"], capture_output=True, text=True)

    # THEN the command fails
    assert result.returncode != 0

    # AND the error message suggests running `clyde setup`
    assert "clyde setup" in result.stderr


def test_setup_installed_clyde(clyde_home):
    # GIVEN an installed Clyde home
    # WHEN calling `which clyde`
    if IS_WINDOWS:
        cmd = "cygpath -w $(which clyde)"
    else:
        cmd = "which clyde"
    proc = run_in_clyde_home(cmd)

    # THEN it returns the clyde binary inside Clyde home
    clyde_path = Path(proc.stdout.strip())
    assert clyde_path == get_bin_path("clyde")
