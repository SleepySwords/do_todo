#!/bin/sh

cargo fmt --all --
cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings
