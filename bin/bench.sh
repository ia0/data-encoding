#!/bin/sh
set -e

fail() { echo "[1;31mError:[m $1"; exit 1; }

DIR="$(mktemp -d)" || fail "Could not create temporary directory."
trap 'rm -rf "$DIR"' 0

TMP="$DIR/tmp"
INPUT="$DIR/input"
OUTPUT="$DIR/output"
ORACLE="$DIR/oracle"
input='"$INPUT"'
output='"$OUTPUT"'

cargo build --release
cp ../target/release/data-encoding "$DIR/de"
PATH="$DIR:$PATH"

head -c16M /dev/urandom > "$INPUT" || fail "Could not create input."

measure() {
  local cmd="time -f '%e %U %S' $1 2>&1"
  [ -z "$2" ] || cmd="$cmd > $2"
  echo "$cmd"
}

sum() {
  local x="$1" y="${1#* }"
  echo "${y% *} + ${x##* }" | bc
}

while true; do
  t="$(time -f '%e' base64 < "$INPUT" 2>&1 > "$OUTPUT")"
  echo "Current time: $t"
  t="${t%?}"
  [ "${t%.*}" -eq 0 ] && [ "${t#0.}" -lt 3 ] || break
  cp "$INPUT" "$TMP"
  cat "$TMP" "$TMP" > "$INPUT"
done; unset t

echo "Test with $(stat -c%s "$INPUT") bytes of input."

compare() {
  local x="$(measure "$1" "$2")" y="$(measure "$3" "$4")" a b ta=0 tb=0
  echo
  echo "[36ma: $x[m"
  echo "[36mb: $y[m"
  for i in $(seq 4); do
    a="$(eval "$x")" || fail "$a"
    mv "$OUTPUT" "$ORACLE"
    b="$(eval "$y")" || fail "$b"
    echo "  a: $a;  b: $b;"
    a="$(sum "$a")"
    b="$(sum "$b")"
    ta="$(echo "$ta + $a" | bc)"
    tb="$(echo "$tb + $b" | bc)"
    diff -q "$OUTPUT" "$ORACLE" >/dev/null || fail "Wrong output!"
  done
  echo "[1;36m(b-a)/a: $(echo "($tb - $ta) * 100 / $ta" | bc)%[m"
}

compare "base64 $input" "$output" \
  "de --mode=encode --base=64 --wrap=76 --input=$input --output=$output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -d < $input" "$output" \
  "de --mode=decode --base=64 --skip --input=$input --output=$output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -w 0 < $input" "$output" \
  "de --mode=encode --base=64 < $input" "$output"
compare "base64 -w 0 $input" "$output" \
  "de --mode=encode --base=64 --input=$input --output=$output" ""
BASE64='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
compare "base64 -w 0 $input" "$output" \
  "de --mode=encode --base=custom --symbols=$BASE64 --padding== --input=$input \
--output=$output" ""
cp "$ORACLE" "$INPUT"
compare "base64 -d < $input" "$output" \
  "de --mode=decode --base=64 --input=$input --output=$output" ""
