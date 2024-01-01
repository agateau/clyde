#!/bin/bash
DEMO_DIR=$HOME/.clydedemo
OUT_DIR=$DEMO_DIR/out
rm -rf $OUT_DIR
mkdir $OUT_DIR

# Copy in $HOME but hide demo.nash so that the `ls` in .demo.nash only shows
# the tar.gz.
# We must copy demo.nash because `nash` changes to the dir containing the .nash
# file.
cp $DEMO_DIR/clyde-*.tar.gz .
cp $DEMO_DIR/demo.nash .demo.nash

export PATH=$HOME/.local/bin:$PATH
asciinema rec $OUT_DIR/demo.cast -c "nash .demo.nash"
