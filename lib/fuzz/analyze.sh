#!/bin/sh

cargo run --manifest-path=fuzz/Cargo.toml --release --example=analyze -- "$@"
