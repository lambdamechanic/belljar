#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "Installing cargo-llvm-cov..." >&2
  rustup component add llvm-tools-preview
  cargo install cargo-llvm-cov
fi

echo "Running coverage for workspace..." >&2
cargo llvm-cov --workspace --all-features --fail-under-lines 0 --text --ignore-filename-regex '(.*/tests/|.*/examples/)' "$@"

echo "Writing lcov report to target/llvm-cov/lcov.info..." >&2
mkdir -p target/llvm-cov
cargo llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/lcov.info --ignore-filename-regex '(.*/tests/|.*/examples/)' >/dev/null
echo "Done. See text summary above; lcov at target/llvm-cov/lcov.info" >&2
