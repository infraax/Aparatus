# scripts/

Thin wrappers around what already works. No new daemon, no new service. Run from anywhere; paths are resolved
from the repo root. Linux x86_64 (the IC binaries are x86_64-linux). Needs `cargo`, `curl`, `gunzip`, `sha256sum`.

## Order

```bash
scripts/ic-up.sh       # 1. local IC replica (stock binaries, verified)
scripts/dev-up.sh      # 2. apparatusd for this repo; prints the socket path
target/debug/apparatus rws ic status   # 3. read the replica, record its height on the chain (via apparatusd)
scripts/dev-down.sh    # stop apparatusd
scripts/ic-down.sh     # stop the replica
scripts/dev-check.sh   # the four gates, any time
```

## What each script does

| Script | Does | Refuses to |
|---|---|---|
| `ic-up.sh` | Downloads the stock `release-2026-09-25_03-28-base` binaries (`replica`, `ic-prep`, `orchestrator`, `ic-regedit`, sandboxes) into `.ic-local/bin`, checks `SHA256SUMS` against a pinned digest and every `.gz` against `SHA256SUMS`, keeps the `.gz` so later runs re-verify without downloading. Bootstraps N=1 with `ic-prep` once (subnet index 0 = `SubnetType::System`, root, f=0, no chain keys). Writes `replica.json5` once from `replica --print-sample-config` with only paths, CSP vault, transport port and adapter sockets changed. Starts `replica` with `--replica-version` and `--guestos-version`, waits for `http://127.0.0.1:8080/api/v2/status`. | Build or patch `dfinity/ic`; use dfx, PocketIC or icp-cli; run binaries that fail verification (it re-downloads, then refuses if they still fail); start a second replica when one is running or something else answers on port 8080; start the orchestrator (it needs IPv6, see NAP-corpus `docs/ic/POC-NOTES.md`). |
| `ic-down.sh` | SIGTERM to the replica started by `ic-up.sh`, waits until it is gone. State stays in `.ic-local/run`; the next `ic-up.sh` resumes from it. | Touch any other process. |
| `dev-up.sh` | Builds `apparatus` and `apparatusd` (`--locked`), runs `rws init --solo` once if `.apparatus/` is empty, starts `apparatusd --project <repo>`, prints the socket path (`.apparatus/apparatusd.sock`, mode 0600). | Start a second daemon (prints the running one instead). |
| `dev-down.sh` | SIGTERM to that `apparatusd` (finishes queued requests, removes the socket). | Touch any other process. |
| `quarantine.sh` | Starts apparatusd if needed (`dev-up.sh`), then `rws quarantine` over the socket: records every lockfile pin (hash, age) as a measured `event_envelope`, keeps versions younger than 5 days `waiting`, refuses hash mismatches on the chain (exit 2), writes Advice for feed matches. See `docs/milestones/QUARANTINE.md`. | Write or update any lockfile; apply an advisory; skip the wait silently (`--wait-days` needs `--reason` and is recorded). |
| `dev-check.sh` | `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --workspace --locked`. Stops at the first failure. | — |

`rws ic status` (not a script): reads `http://127.0.0.1:8080/api/v2/status` through ic-agent and writes one measured
`event_envelope` (evidence B) with the certified height and health; replica down → refusal IC-01 on the chain, exit 2.
See `docs/milestones/IC-STATUS.md`.

`.apparatus/` and `.ic-local/` are git-ignored. Ports used: `8080/tcp` (replica public API), `2497` (replica transport).
Set `IC_DIR=` to put the replica elsewhere. Full manual runbook: NAP-corpus `docs/ic/LOCAL-RUN.md`.
