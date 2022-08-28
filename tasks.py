"""
SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>

SPDX-License-Identifier: GPL-3.0-or-later

A set of tasks to simplify the release process. See docs/release-check-list.md
for details.
"""

import os
import re
import shutil
import sys

from pathlib import Path
from typing import List

from invoke import task, run


ARTIFACTS_DIR = Path("artifacts")


def get_version():
    return os.environ["VERSION"]


def erun(*args, **kwargs):
    """Like run, but with echo on"""
    kwargs["echo"] = True
    return run(*args, **kwargs)


def cerun(c, *args, **kwargs):
    """Like Context.run, but with echo on"""
    kwargs["echo"] = True
    return c.run(*args, **kwargs)


def ask(msg: str) -> str:
    """Show a message, wait for input and returns it"""
    print(msg, end=" ")
    return input()


def is_ok(msg: str) -> bool:
    """Show a message, append (y/n) and return True if user select y or Y"""
    answer = ask(f"{msg} (y/n)").lower()
    return answer == "y"


@task
def create_pr(c):
    """Create a pull-request and mark it as auto-mergeable"""
    result = cerun(c, "gh pr create --fill", warn=True)
    if not result:
        sys.exit(1)
    cerun(c, f"gh pr merge --auto -dm")


@task
def update_version(c):
    version = get_version()
    path = Path("Cargo.toml")
    text = path.read_text()
    text, count = re.subn(r"^version = .*", f"version = \"{version}\"", text,
                          flags=re.MULTILINE)
    assert count == 0 or count == 1
    path.write_text(text)


@task
def prepare_release(c):
    version = get_version()
    run(f"gh issue list -m {version}", pty=True)
    run("gh pr list", pty=True)
    if not is_ok("Continue?"):
        sys.exit(1)

    erun("git checkout main")
    erun("git pull")
    erun("git status -s")
    if not is_ok("Continue?"):
        sys.exit(1)

    prepare_release2(c)


@task
def prepare_release2(c):
    version = get_version()
    erun("git checkout -b prep-release")

    update_version(c)

    erun(f"changie batch {version}")
    print(f"Review/edit changelog (.changes/{version}.md)")
    if not is_ok("Looks good?"):
        sys.exit(1)
    erun("changie merge")
    print("Review CHANGELOG.md")

    if not is_ok("Looks good?"):
        sys.exit(1)

    prepare_release3(c)


@task
def prepare_release3(c):
    version = get_version()
    erun("git add Cargo.toml Cargo.lock CHANGELOG.md .changes")
    erun(f"git commit -m 'Prepare {version}'")
    erun("git push -u origin prep-release")

    erun("cargo publish --dry-run --allow-dirty")
    erun("cargo package --list --allow-dirty")

    # `publish --dry-run` updates Cargo.lock. Commit the changes.
    erun("git add Cargo.lock")
    create_pr(c)


@task
def tag(c):
    version = get_version()
    erun("git checkout main")
    erun("git pull")
    if not is_ok("Create tag?"):
        sys.exit(1)

    erun(f"git tag -a {version} -m 'Releasing version {version}'")

    erun("git push")
    erun("git push --tags")


def get_artifact_list() -> List[Path]:
    assert ARTIFACTS_DIR.exists()
    return list(ARTIFACTS_DIR.glob("*.tar.gz")) + list(ARTIFACTS_DIR.glob("*.zip"))


@task
def download_artifacts(c):
    if ARTIFACTS_DIR.exists():
        shutil.rmtree(ARTIFACTS_DIR)
    ARTIFACTS_DIR.mkdir()
    erun(f"gh run download --dir {ARTIFACTS_DIR}", pty=True)


@task
def publish(c):
    version = get_version()
    files_str = " ".join(str(x) for x in get_artifact_list())
    erun(f"gh release create {version} -F.changes/{version}.md {files_str}")
    erun("cargo publish")


@task
def update_store(c):
    version = get_version()
    with c.cd("../clyde-store"):
        cerun(c, "git checkout main")
        cerun(c, "git pull")
        cerun(c, "git checkout -b update-clyde")
        cerun(c, f"clydetools fetch clyde.yaml", pty=True)
        cerun(c, "git add clyde.yaml")
        cerun(c, f"git commit -m 'Update clyde to {version}'")
        cerun(c, "git push -u origin update-clyde")
        create_pr(c)
