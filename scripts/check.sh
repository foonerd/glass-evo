#!/bin/bash
# The workshop pass CI runs: formatting, lints with warnings denied, tests,
# and the documentation with warnings denied. Run it before a commit.
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"

echo "check: format"
cargo fmt --all -- --check

echo "check: clippy"
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "check: browser module"
# The face and its module for a browser, linted and built for their target.
cargo clippy -p face -p glass-evo-face --target wasm32-unknown-unknown --locked -- -D warnings
cargo build -p glass-evo-face --profile face --target wasm32-unknown-unknown --locked

echo "check: tests"
cargo test --workspace --locked

echo "check: documentation"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

echo "check: shell scripts"
for script in scripts/*.sh; do
  sh -n "$script" || { echo "check: $script does not parse" >&2; exit 1; }
done

echo "check: clean"
