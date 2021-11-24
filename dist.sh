#!/usr/bin/env bash

set -e

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

git pull

./clippy.sh
./example.sh > example.out

npx @rmw/md-include .markdown.json

cargo set-version --bump patch

git add -u
git commit -m dist
git push

cargo +nightly publish

