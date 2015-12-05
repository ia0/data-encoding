#!/bin/sh
set -e

fail() { echo "[1;31mError:[m $1"; exit 1; }

PATH="$(readlink -f target/release/examples):$PATH"

DIR="$(mktemp -d)" || fail "Could not create temporary directory."
trap 'rm -rf "$DIR"' 0

TMP="$DIR/tmp"
INPUT="$DIR/input"
OUTPUT="$DIR/output"
ORACLE="$DIR/oracle"
input='"$INPUT"'
output='"$OUTPUT"'

head -c16M /dev/urandom > "$INPUT" || fail "Could not create input."

measure() {
  local cmd="time -f '%e %U %S' $1 2>&1"
  [ -z "$2" ] || cmd="$cmd > $2"
  echo "$cmd"
}

while true; do
  t="$(time -f '%e' base64 < "$INPUT" 2>&1 > "$OUTPUT")"
  echo "Current time: $t"
  t="${t%?}"
  [ "${t%.*}" -eq 0 -a "${t#0.}" -lt 3 ] || break
  cp "$INPUT" "$TMP"
  cat "$TMP" "$TMP" > "$INPUT"
done; unset t

echo "Test with $(stat -c%s "$INPUT") bytes of input."

compare() {
  local x="$(measure "$1" "$2")" y="$(measure "$3" "$4")" a b
  echo
  echo "a: $x"
  echo "b: $y"
  for i in $(seq 4); do
    a="$(eval "$x")" || fail "$a"
    mv "$OUTPUT" "$ORACLE"
    b="$(eval "$y")" || fail "$b"
    echo "  a: $a;  b: $b;"
    diff -q "$OUTPUT" "$ORACLE" >/dev/null || fail "Wrong output!"
  done
}

compare "base64 $input" "$output" "encode -w76 -i $input -o $output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -d < $input" "$output" "encode -d -s -i $input -o $output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -w 0 < $input" "$output" "encode < $input" "$output"
compare "base64 -w 0 $input" "$output" "encode -i $input -o $output" ""
compare "base64 -w 0 $input" "$output" "encode -b=ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/ -i $input -o $output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -d < $input" "$output" "encode -d -i $input -o $output" ""
