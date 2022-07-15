#!/bin/bash

# SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

export CLYDE_HOME=/tmp/test-clyde

rm -rf $CLYDE_HOME

cargo build

CLYDE_CMD="target/debug/clyde"

# Running a command with a non existing CLYDE_HOME should cause Clyde to print
# a message prompting the user to run `clyde setup`
$CLYDE_CMD update > /tmp/out 2>&1 || true

if grep -q 'clyde setup' /tmp/out ; then
    echo OK
else
    echo FAIL
    exit 1
fi
