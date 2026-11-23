#!/usr/bin/env bash
# Lines of engine/ after rustfmt, code and tests apart (so formatting and moving code score nothing).
#   scripts/engine-lines.sh [ref]   (default: the working tree, uncommitted edits included). Prints "code tests".
set -euo pipefail
root=$(git rev-parse --show-toplevel); cd "$root"
ref=${1:-$(git stash create)}; ref=${ref:-HEAD}
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
git archive "$ref" engine | tar -x -C "$tmp"
export PATH=/d/rust/cargo/bin:$PATH RUSTUP_HOME=D:/rust/rustup CARGO_HOME=D:/rust/cargo
if rustfmt --version >/dev/null 2>&1; then
  find "$tmp/engine" -name '*.rs' -print0 | xargs -0 rustfmt --edition 2024
else
  echo "engine-lines: rustfmt is not installed; counting unformatted lines" >&2
fi
find "$tmp/engine" -name '*.rs' | perl "$root/scripts/engine-lines.pl"
