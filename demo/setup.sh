#!/usr/bin/env bash
set -euo pipefail

VERSION=0.6.0

cd $(dirname $0)
docker build -t clydedemo .
curl -LO \
    --continue-at - \
    https://github.com/agateau/clyde/releases/download/$VERSION/clyde-$VERSION-x86_64-linux.tar.gz

rm -rf clyde-$VERSION

docker run -it --rm -v $PWD:/home/demo/.clydedemo clydedemo:latest
