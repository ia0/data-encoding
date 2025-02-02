#!/bin/sh

N="$(cargo fuzz list | wc -l)"
i=1
next() { cargo fuzz list | head -n$i | tail -n1; }
while cargo fuzz run "$(next)" -- -max_total_time=600; do
  i=$(( i % N + 1 ))
done
