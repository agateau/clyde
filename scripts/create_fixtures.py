#!/usr/bin/env python
"""
SPDX-FileCopyrightText: 2026 Aurélien Gâteau <mail@agateau.com>

SPDX-License-Identifier: GPL-3.0-or-later
"""

import tarfile
from pathlib import Path
from tarfile import TarInfo

FIXTURES_DIR = Path(__file__).parent / "../src/fixtures"


def create_symlink(name: str, target: str) -> TarInfo:
    info = TarInfo(name)
    info.type = tarfile.SYMTYPE
    info.linkname = target
    return info


def create_evil_link():
    out = tarfile.open(FIXTURES_DIR / "evil_link.tar.gz", mode="w:gz")
    try:
        out.addfile(create_symlink(name="dir", target="../dir"))
        out.addfile(TarInfo("dir/payload"))
    finally:
        out.close()


def create_ok_link():
    out = tarfile.open(FIXTURES_DIR / "ok_link.tar.gz", mode="w:gz")
    try:
        out.addfile(create_symlink(name="dir1/symlink", target="../dir2/real_file"))
        out.addfile(TarInfo("dir2/real_file"))
        out.addfile(create_symlink(name="dir_symlink", target="dir2"))

    finally:
        out.close()


def main():
    create_evil_link()
    create_ok_link()


if __name__ == "__main__":
    main()
