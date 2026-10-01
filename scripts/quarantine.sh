#!/usr/bin/env bash
# Package quarantine over the apparatusd socket: starts the daemon if needed (dev-up.sh), then
# `apparatus rws quarantine` for this repo. Extra arguments are passed through
# (e.g. --advisory-feed FILE, --lockfile FILE, --wait-days N --reason TEXT).
# Never writes a lockfile. Exit 2 = a refusal was recorded on the chain.
set -euo pipefail
source "$(dirname "$0")/common.sh"
"$ROOT/scripts/dev-up.sh" > /dev/null
exec "$ROOT/target/debug/apparatus" --project "$ROOT" rws quarantine "$@"
