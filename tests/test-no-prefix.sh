#!/bin/bash
set -euo pipefail

export CLYDE_PREFIX=/tmp/test-clyde

rm -rf $CLYDE_PREFIX

cargo build

CLYDE_CMD="target/debug/clyde"

# Running a command on an non existing prefix should cause Clyde to print a
# message prompting the user to run `clyde setup`
$CLYDE_CMD update > /tmp/out 2>&1 || true

if grep -q 'clyde setup' /tmp/out ; then
    echo OK
else
    echo FAIL
    exit 1
fi
