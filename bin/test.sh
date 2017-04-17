#!/bin/sh
set -e

run() {
  local cmd="$1" in="$2" tout="$3" terr="$4" cmd sout serr code=0
  sout="$(echo -n "$in" | eval "$cmd" 2>/dev/null)" || true
  serr="$(echo -n "$in" | eval "$cmd" 2>&1 >/dev/null)" || code=$?
  [ "$tout" = "$sout" ] || { echo out; return 1; }
  [ "$terr" = "$serr" ] || { echo err; return 1; }
  if [ "$code" -eq 0 ]; then
    [ -z "$terr" ] || { echo code; return 1; }
  else
    [ "$code" -eq 1 ] || { echo code; return 1; }
  fi
}

count=0
error=0

unit() {
  local opts="$1" in="$2" out="$3" err="$4" cmd res
  [ -z "$err" ] || err="data-encoding: $err"
  cmd="../target/debug/data-encoding $opts"
  count=$(expr $count + 1)
  if res=$(run "$cmd" "$in" "$out" "$err"); then
    echo -n " [32m$count[m"
  else
    echo " [1;31m$count[m($res)"
    echo "echo '$in' | $cmd"
    echo -n "$in" | eval "$cmd" || true
    echo
    error=$(expr $error + 1)
  fi
}

cargo build

echo -n 'encode:'
unit "--mode=encode --base=64"
unit "--mode=encode --base=64" f Zg==
unit "--mode=encode --base=64" fo Zm8=
unit "--mode=encode --base=64" foo Zm9v
unit "--mode=encode --base=64" foob Zm9vYg==
unit "--mode=encode --base=64" fooba Zm9vYmE=
unit "--mode=encode --base=64" foobar Zm9vYmFy
echo
echo -n 'decode:'
unit "--mode=decode --base=64"
unit "--mode=decode --base=64" Zg== f
unit "--mode=decode --base=64" Zm8= fo
unit "--mode=decode --base=64" Zm9v foo
unit "--mode=decode --base=64" Zm9vYg== foob
unit "--mode=decode --base=64" Zm9vYmE= fooba
unit "--mode=decode --base=64" Zm9vYmFy foobar
unit "--mode=decode_concat --base=64" Zg==Zm8=Zg== ffof
echo
echo -n 'wrap:'
unit "--mode=encode --base=64 --wrap=0" fo '' 'Invalid wrap value'
unit "--mode=encode --base=64 --wrap=1" fo "$(printf 'Z\nm\n8\n=')"
unit "--mode=encode --base=64 --wrap=2" fo "$(printf 'Zm\n8=')"
unit "--mode=encode --base=64 --wrap=3" fo "$(printf 'Zm8\n=')"
echo
echo -n 'skip:'
unit "--mode=decode --base=64 --skip" "$(printf 'Z\nm\n\n\n\n8=')" fo
unit "--mode=decode_concat --base=64 --skip" "$(printf 'Zg\n\n=\n\n=Z\n\nm\n\n8=\nZg-=')" '' 'invalid symbol at 19'
unit "--mode=decode_concat --base=64 --skip --block=8" "$(printf 'Zg\n\n=\n\n=Z\n\nm\n\n8=\nZg-=')" ffo 'invalid symbol at 19'
echo
echo -n 'symbol:'
unit "--mode=decode_concat --base=64" Zg==Zm8=Zg-= '' 'invalid symbol at 10'
unit "--mode=decode_concat --base=64 --block=8" Zg==Zm8=Zg-= ffo 'invalid symbol at 10'
unit "--mode=decode --base=64" "$(printf 'Z\ng=')" '' 'invalid symbol at 1'
unit "--mode=decode --base=64" "$(printf 'Z g=')" '' 'invalid symbol at 1'
unit "--mode=decode --base=64" "$(printf 'Z=g=')" '' 'invalid symbol at 1'
unit "--mode=decode --base=64" "$(printf 'Zm9vZm9v----')" '' 'invalid symbol at 8'
unit "--mode=decode --base=64 --block=8" "$(printf 'Zm9vZm9v----')" foofoo 'invalid symbol at 8'
echo
echo -n 'padding:'
unit "--mode=decode --base=64" "$(printf 'Z===')" '' 'invalid padding length at 1'
echo
echo -n 'length:'
unit "--mode=decode --base=64" Zg= '' 'invalid length at 0'
unit "--mode=decode --base=64" Zg==Z 'f' 'invalid length at 4'
unit "--mode=decode --base=64" Zg==Zg 'f' 'invalid length at 4'
unit "--mode=decode --base=64" Zg==ZgZ 'f' 'invalid length at 4'
echo
echo -n 'trailing:'
unit "--mode=decode --base=64" Zh== '' 'non-zero trailing bits at 1'
echo
echo -n 'custom:'
unit "--mode=info --base=custom --symbols=0" '' '' 'invalid number of symbols'
unit "--mode=info --base=custom --symbols=$(printf '\303\251')" '' '' 'non-ascii symbol 0xc3'
unit "--mode=info --base=custom --symbols=01 --translate=$(printf '\303\251')" '' \
     "$(printf 'symbols: "01"\nbit_order: MostSignificantFirst')"
echo

[ "$error" -eq 0 ]
