#!/bin/sh

for target in $(cargo fuzz list); do
  cargo fuzz run $target -- -runs=0
done
