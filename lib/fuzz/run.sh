#!/bin/sh

LIST="$*"
[ -n "$LIST" ] || LIST=$(echo $(cargo fuzz list))
list() { for x in $LIST; do echo $x; done; }

N="$(list | wc -l)"
i=1
next() { list | head -n$i | tail -n1; }
while cargo fuzz run "$(next)" -- -max_total_time=600; do
  i=$(( i % N + 1 ))
done
