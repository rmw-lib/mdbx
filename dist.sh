#!/usr/bin/env bash

set -e

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

git pull

./clippy.sh
./example.sh > example.out

npx @rmw/md-include .markdown.json

cargo set-version --bump patch

version=$(cat Cargo.toml|grep "^version"|awk -F\" '{print $2}')

git add -u
git commit -m "v$version"
git tag v$version
git push

cargo +nightly publish

