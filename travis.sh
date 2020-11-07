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

bench_lib() {
  info_exec cargo lib bench --verbose
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

bench_bin() {
  info_exec bin ./bench.sh
}

test_outdated() {
  which cargo-outdated >/dev/null || info_exec cargo install cargo-outdated
  # Workaround error: failed to parse lock file at: data-encoding/Cargo.lock
  # Caused by: invalid serialized PackageId for key `package.dependencies`
  info_exec git clean -fxd
  info_exec cargo outdated -w -R --exit-code=1
}

send_coverage() {
  which cargo-tarpaulin >/dev/null || info_exec cargo install cargo-tarpaulin
  info_exec git clean -fxd
  # We have to give an explicit list of --exclude-files due to
  # https://github.com/xd009642/tarpaulin/issues/394
  info_exec lib cargo tarpaulin \
    --ciserver=travis-ci --coveralls="$TRAVIS_JOB_ID" \
    --exclude-files='../*' --exclude-files=fuzz --exclude-files=tests
}

if [ -n "$TRAVIS_JOB_ID" ]; then
  info_exec git clean -fxd
  test_lib
  if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    bench_lib
    test_nostd
  fi
  test_bin
  bench_bin
  test_outdated
  if [ "$TRAVIS_RUST_VERSION" = stable ]; then
    send_coverage || true
  fi
else
  test_lib +stable
  test_lib +nightly
  test_nostd +nightly
  test_bin +stable
  test_bin +nightly
fi

info "Done"
