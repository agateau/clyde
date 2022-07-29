"""
SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>

SPDX-License-Identifier: GPL-3.0-or-later

A set of tasks to simplify the release process. See docs/release-check-list.md
for details.
"""

import json
import os
import re
import shutil
import sys

from http.client import HTTPResponse
from pathlib import Path
from typing import List, Dict
from urllib import request

from invoke import task, run


ARTIFACTS_DIR = Path("artifacts")


def get_version():
    return os.environ["VERSION"]


def create_request(url: str, headers: Dict[str, str]) -> request.Request:
    req = request.Request(url)
    for key, value in headers.items():
        req.add_header(key, value)

    return req


def http_get(url: str, headers: Dict[str, str]) -> HTTPResponse:
    req = create_request(url, headers)
    return request.urlopen(req)


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
    def extract_pr_id(text):
        match = re.search(r"/pull/(\d+)", text)
        if not match:
            print(f"Can't find pull request ID from:\n'''\n{text}\n'''")
            sys.exit(1)
        return match.group(1)

    result = cerun(c, "gh pr create --fill", warn=True)
    if result:
        pr_id = extract_pr_id(result.stdout)
    elif "a pull request for branch" in result.stderr:
        # PR already opened, PR ID is in stderr
        pr_id = extract_pr_id(result.stderr)
    else:
        sys.exit(1)

    print(f"Pull request ID: {pr_id}")
    cerun(c, f"gh pr merge --auto -dm {pr_id}")


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
    tag_url = f"https://api.github.com/repos/agateau/clyde/releases/tags/{version}"
    print(f"Fetching release info from {tag_url}")
    response = http_get(tag_url, dict())
    dct = json.load(response)
    archives_url = [x["browser_download_url"] for x in dct["assets"]]

    with c.cd("../clyde-store"):
        cerun(c, "git checkout main")
        cerun(c, "git pull")
        cerun(c, "git checkout -b update-clyde")
        urls_str = " ".join(archives_url)
        cerun(c, f"clydetools add-build clyde.yaml {version} {urls_str}")
        cerun(c, "git add clyde.yaml")
        cerun(c, f"git commit -m 'Update clyde to {version}'")
        cerun(c, "git push -u origin update-clyde")
        create_pr(c)
