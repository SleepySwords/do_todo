#!/bin/sh

cargo fmt --all --
cargo clippy --fix --allow-dirty
