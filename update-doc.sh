#!/bin/sh
set -e

fail() { echo "[1;31mError:[m $1"; exit 1; }

CUR_BRANCH="$(git symbolic-ref -q HEAD)"
[ "$CUR_BRANCH" = refs/heads/master ] || fail "Not on master"

CUR_STATUS="$(git status --porcelain)"
[ -z "$CUR_STATUS" ] || fail "Dirty working directory"
git clean -fxd

CUR_TAGS="$(git tag --points-at HEAD | wc -l)"
[ "$CUR_TAGS" -eq 1 ] || fail "No unique tag"
CUR_TAG="$(git tag --points-at HEAD)"
( cd lib; cargo doc )

DOC_BRANCH=gh-pages
git show-ref -q --verify "refs/heads/$DOC_BRANCH" ||
  fail "$DOC_BRANCH does not exist"
git checkout -q "$DOC_BRANCH"
mv target/doc "$CUR_TAG"
git add "$CUR_TAG"
git commit -qm"$CUR_TAG"
git checkout -q master
