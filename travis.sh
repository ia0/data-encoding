#!/bin/sh
set -e

info() { echo "[1;36mInfo:[m $1"; }

git clean -fxd

( cd lib
  info "Build library"
  cargo build --verbose

  info "Test library"
  cargo test --verbose

  ( cd macro
    info "Build macro library"
    cargo build --verbose

    info "Test macro library"
    cargo test --verbose
  )

  if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    info "Benchmark library"
    cargo bench --verbose

    ( cd macro
      info "Build macro library (no stable feature)"
      cargo build --no-default-features --verbose

      info "Test macro library (no stable feature)"
      cargo test --no-default-features --verbose
    )
  fi
)
( cd bin
  info "Build binary"
  cargo build --verbose

  info "Test binary"
  cargo test --verbose
  ./test.sh

  info "Benchmark binary"
  ./bench.sh
)
( info "Ensure cargo-outdated is installed"
  which cargo-outdated >/dev/null || cargo install cargo-outdated

  info "Test dependencies"
  cargo outdated -w -R --exit-code=1
)

( [ -n "$TRAVIS_JOB_ID" ] || exit
  git clean -fxd

  info "Download kcov"
  wget -q https://github.com/SimonKagstrom/kcov/archive/master.tar.gz

  info "Build kcov"
  tar xf master.tar.gz
  mkdir kcov-master/build
  ( cd kcov-master/build
    cmake ..
    make >/dev/null
  ) || return

  info "Test library coverage"
  ( cd lib; cargo test --verbose --no-run )
  find target/debug -maxdepth 1 -type f -perm /u+x -printf '%P\n' |
    while IFS= read -r test; do
      info "Run $test"
      ./kcov-master/build/src/kcov --include-path=lib/src/ target/kcov-"$test" \
                                   ./target/debug/"$test"
    done; unset test

  info "Send coverage to coveralls.io"
  ./kcov-master/build/src/kcov --coveralls-id="$TRAVIS_JOB_ID" \
                               --merge target/kcov target/kcov-*
) || true

info "Done"
