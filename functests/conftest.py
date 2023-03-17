# SPDX-FileCopyrightText: 2023 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

import os
import platform
import shutil
import subprocess

from pathlib import Path
from tempfile import TemporaryDirectory

import pytest

from file_utils import rm_rf

ROOT_DIR = Path(__file__).parent.parent.absolute()

IS_WINDOWS = platform.system() == "Windows"

# On GitHub Windows runner one gets an error message from WSL saying no
# distributions are installed when running `bash` as is. Make it possible to
# provide the full path to the bash binary to avoid this.
BASH_CMD = os.environ.get("BASH_CMD", "bash")


def _run(*args, check=True, **kwargs):
    """Wrapper for subprocess.run() to run a command and by default check the
    result but in case of failure, calls pytest.fail() instead of raising an
    exception.
    """
    proc = subprocess.run(
        *args, capture_output=True, text=True, **kwargs
    )
    if check and proc.returncode != 0:
        pytest.fail(f"""Command {args} failed with exit code {proc.returncode}
STDOUT:
{proc.stdout}
STDERR:
{proc.stderr}
""")
    return proc


def run_clyde(*args, check=True):
    """Run a Clyde command"""
    cmd = ["cargo", "run", "-r", "--bin", "clyde", *args]
    return _run(cmd, check=check, cwd=str(ROOT_DIR))


def run_in_clyde_home(cmd, check=True):
    """Run a command inside the Clyde home"""
    CLYDE_HOME = os.environ["CLYDE_HOME"]
    script = f". scripts/activate.sh ; {cmd}"
    cmd = [BASH_CMD, "-c", script]
    # Run the command directly in CLYDE_HOME. This avoids having to deal with
    # cygpath on Windows to get the path to the activate.sh script.
    return _run(cmd, check=check, cwd=str(CLYDE_HOME))


def get_bin_path(bin_name):
    """Returns the path to a binary inside Clyde home.
    Does not check if the binary exists."""
    if IS_WINDOWS:
        bin_name += ".exe"
    return Path(os.environ["CLYDE_HOME"]) / "inst" / "bin" / bin_name


@pytest.fixture(scope="session")
def _setup_clyde_home():
    """Creates a Clyde home and back it up. Used by clyde_home."""
    # Make sure outside environment does not affect tests
    try:
        del os.environ["CLYDE_INST_DIR"]
    except KeyError:
        pass

    with TemporaryDirectory(prefix="clyde-functests") as tmp_dir:
        clyde_home = Path(tmp_dir) / "clyde_home"
        backup_dir = Path(tmp_dir) / "backup"
        os.environ["CLYDE_HOME"] = str(clyde_home)
        run_clyde("setup")
        shutil.move(clyde_home, backup_dir)
        yield clyde_home, backup_dir


@pytest.fixture()
def clyde_home(_setup_clyde_home):
    """Creates a Clyde home by reusing the Clyde home created by
    _setup_clyde_home()"""
    clyde_home, backup_dir = _setup_clyde_home
    rm_rf(clyde_home)
    shutil.copytree(backup_dir, clyde_home)
    yield clyde_home
