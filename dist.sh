#!/usr/bin/env bash

set -e

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

git pull

./clippy.sh

npx @rmw/md-include .markdown.json

#cargo set-version --bump patch

version=v$(cat Cargo.toml|grep "^version"|awk -F\" '{print $2}')

#git add -u
#git commit -m $version
#git tag v$version
git push v$version -f
git push -f

cargo +nightly publish

