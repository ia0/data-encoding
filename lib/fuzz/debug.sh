#!/bin/sh

cargo run --manifest-path=fuzz/Cargo.toml --example=debug -- "$1"
