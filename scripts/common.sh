# Shared settings for scripts/*.sh. Sourced, not run.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Stock IC release (Path B): release-2026-09-25_03-28-base. See NAP-corpus docs/ic/LOCAL-RUN.md.
IC_COMMIT=d26cd031176beec51b39fbb9e39e80a3a46a748e
IC_SUMS_SHA256=f3d88fe59a0895a3408f94a27c03691f237c36dd4e2a6e4e627b1a3d18c7e606   # sha256 of SHA256SUMS
IC_BASE=https://download.dfinity.systems/ic/$IC_COMMIT/binaries/x86_64-linux
IC_BINS="replica ic-prep orchestrator ic-regedit canister_sandbox sandbox_launcher compiler_sandbox"
IC_DIR="${IC_DIR:-$ROOT/.ic-local}"
IC_STATUS_URL=http://127.0.0.1:8080/api/v2/status
APP_DIR="$ROOT/.apparatus"
