#!/usr/bin/env bash

set -o errexit
set -o xtrace

cargo build
cargo build --target x86_64-pc-windows-gnu
cargo test
cargo doc
