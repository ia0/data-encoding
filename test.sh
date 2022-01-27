#!/bin/sh
set -e

info() {
  echo "[1;36m$1[m"
}

info_exec() {
  local dir="$1"; shift
  info "( cd $dir && $* )"
  ( cd "${dir}" && "$@" )
}

test_lib() {
  info_exec lib cargo $1 build --verbose
  info_exec lib cargo $1 test --verbose
  info_exec lib cargo $1 build --verbose --no-default-features
  info_exec lib cargo $1 build --verbose --no-default-features --features=alloc
  info_exec lib/macro cargo $1 build --verbose
  info_exec lib/macro cargo $1 test --verbose
}

test_nostd() {
  info_exec nostd cargo $1 run --verbose --release
  info_exec nostd cargo $1 run --verbose --release --features=alloc
}

test_bin() {
  info_exec bin cargo $1 build --verbose
  info_exec bin cargo $1 test --verbose
  info_exec bin ./test.sh $1
}

test_lib +1.46
test_lib +stable
test_lib +nightly
test_nostd +nightly
test_bin +stable
test_bin +nightly

info "Done"
