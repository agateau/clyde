# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

import os
import sqlite3
from pathlib import Path

from conftest import run_clyde, IS_WINDOWS, get_bin_path


def _get_db_connection():
    db_path = Path(os.environ["CLYDE_HOME"], "clyde.sqlite")
    assert db_path.exists(), db_path
    return sqlite3.connect(db_path)


def set_package_version(name: str, version: str) -> None:
    conn = _get_db_connection()
    conn.execute(
        """update installed_package
        set installed_version=?
        where name=?""",
        (version, name),
    )
    conn.commit()


def add_installed_package(name: str, version: str) -> None:
    conn = _get_db_connection()
    conn.execute(
        """insert into installed_package(name, installed_version, requested_version)
        values(?, ?, '*')""",
        (name, version),
    )
    conn.commit()


def get_package_version(name: str) -> str:
    conn = _get_db_connection()
    res = conn.execute(
        """select installed_version
        from installed_package
        where name=?""",
        (name,),
    )
    return res.fetchone()[0]


def test_upgrade_install_clyde_only(clyde_home):
    # GIVEN a Clyde home
    # AND an outdated clyde install
    set_package_version("clyde", "0.1.0")

    # AND an outdated starship install
    add_installed_package("starship", "0.1.0")

    # WHEN `clyde upgrade` runs
    result = run_clyde("upgrade")

    # THEN it only upgrades clyde
    assert get_package_version("clyde") != "0.1.0"
    assert get_package_version("starship") == "0.1.0"

    # AND on Windows, the old clyde.exe has been renamed to _clyde.exe
    if IS_WINDOWS:
        _clyde_exe = get_bin_path("_clyde")
        assert _clyde_exe.exists()
