#!/bin/sh
set -e

info() {
  echo -n "[1;36mInfo:[m $1"
  if [ -n "$TOOLCHAIN" ]; then
    echo -n " ($TOOLCHAIN)"
  fi
  echo
}

test_lib() {
  ( cd lib
    [ -n "$1" ] && TOOLCHAIN="${1#+}"

    info "Build library"
    cargo $1 build --verbose

    info "Test library"
    cargo $1 test --verbose

    info "Build noalloc library"
    cargo $1 build --verbose --no-default-features

    info "Build nostd library"
    cargo $1 build --verbose --no-default-features --features=alloc

    ( cd macro
      info "Build macro library"
      cargo $1 build --verbose

      info "Test macro library"
      cargo $1 test --verbose
    )
  )
}

bench_lib() {
  ( cd lib

    info "Benchmark library"
    cargo bench --verbose
  )
}

test_nostd() {
  ( cd nostd
    [ -n "$1" ] && TOOLCHAIN="${1#+}"

    info "Test noalloc binary"
    cargo $1 run --verbose --release

    info "Test nostd binary"
    cargo $1 run --verbose --release --features=alloc
  )
}

test_bin() {
  ( cd bin
    [ -n "$1" ] && TOOLCHAIN="${1#+}"

    info "Build binary"
    cargo $1 build --verbose

    info "Test binary"
    cargo $1 test --verbose
    ./test.sh $1
  )
}

bench_bin() {
  ( cd bin

    info "Benchmark binary"
    ./bench.sh
  )
}

test_outdated() {
  ( info "Ensure cargo-outdated is installed"
    which cargo-outdated >/dev/null || cargo install cargo-outdated

    # Workaround error: failed to parse lock file at: data-encoding/Cargo.lock
    # Caused by: invalid serialized PackageId for key `package.dependencies`
    git clean -fxd

    info "Test dependencies"
    cargo outdated -w -R --exit-code=1
  )
}

send_coverage() {
  ( git clean -fxd

    info "Install tarpaulin"
    which cargo-tarpaulin >/dev/null || cargo install cargo-tarpaulin

    info "Test and send library coverage to coveralls.io"
    ( cd lib
      # We have to give an explicit list of --exclude-files due to
      # https://github.com/xd009642/tarpaulin/issues/394
      cargo tarpaulin --ciserver travis-ci --coveralls "$TRAVIS_JOB_ID" \
        --exclude-files '../*' --exclude-files fuzz --exclude-files tests
    )
  ) || true
}

if [ -n "$TRAVIS_JOB_ID" ]; then
  git clean -fxd
  test_lib
  if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    bench_lib
    test_nostd
  fi
  test_bin
  bench_bin
  test_outdated
  if [ "$TRAVIS_RUST_VERSION" = stable ]; then
    send_coverage
  fi
else
  test_lib +stable
  test_lib +nightly
  test_nostd +nightly
  test_bin +stable
  test_bin +nightly
fi

info "Done"
