#!/bin/bash
set -exuo pipefail

export CLYDE_PREFIX=/tmp/test-clyde

rm -rf $CLYDE_PREFIX

clyde setup
clyde update
clyde install gh@=2.11.3
clyde show gh
clyde remove gh
clyde install gh

. $CLYDE_PREFIX/scripts/activate.sh

which gh

gh --version
