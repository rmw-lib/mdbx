#!/usr/bin/env bash

set -e
_DIR=$(dirname $(realpath "$0"))
cd $_DIR

cargo build --example main
./target/debug/examples/main
