#!/usr/bin/env bash
# The four gates. Stops at the first failure.
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$ROOT"
echo "== cargo test --workspace";                                   cargo test --workspace
echo "== cargo clippy --workspace --all-targets -- -D warnings";    cargo clippy --workspace --all-targets -- -D warnings
echo "== cargo fmt --all -- --check";                               cargo fmt --all -- --check
echo "== cargo build --workspace --locked";                         cargo build --workspace --locked
echo "all four gates passed"
