#!/bin/bash
set -exuo pipefail

export CLYDE_HOME=/tmp/test-clyde

rm -rf $CLYDE_HOME

cargo build

CLYDE_CMD="cargo run --bin clyde"

$CLYDE_CMD setup
$CLYDE_CMD update
$CLYDE_CMD install gh@=2.13.0
$CLYDE_CMD show gh
$CLYDE_CMD remove gh
$CLYDE_CMD install gh

. $CLYDE_HOME/scripts/activate.sh

which gh

gh --version
