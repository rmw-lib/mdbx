#!/usr/bin/env bash

DIR=$(cd "$(dirname "$0")"; pwd)
set -ex
cd $DIR

npx md-padding ./readme.make.md

npx @rmw/md-include .markdown.json

cp README.md ../blog-vuepress2/site/zh/log/2021-12-21-mdbx.md
