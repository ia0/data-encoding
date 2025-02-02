#!/bin/sh

for target in $(cargo fuzz list); do
  cargo fuzz cmin $target
done
